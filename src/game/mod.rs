mod map;
mod entity;
mod player;

use std::fmt::Display;
use std::fmt;
use std::ptr;
use itertools::Itertools;

// Flat structure for AI
// TODO: probably GameView should do this as it will be the main API for AI
pub use game::map::{Map, Direction, Coordinates, Location, Tile};
pub use game::entity::{Entity, EntityType, EntitiesIter, Unit, Building, Resources, Entities, EntityID};
pub use game::player::{Player, Colour, AI, EmptyPersistentState, Owned};
use game_view::GameView;

#[derive(Debug)]
pub struct Game<'p, 'm> {
    name: String,
    round: u32,
    // TODO: hide begind GameView
    pub map: &'m Map,
    pub entities: Entities<'p, 'm>,
}

#[derive(Debug)]
pub enum GameRuleViolation<'p: 'e, 'm: 'e, 'e> {
    BadEntityPlacement,
    InvalidMove(&'e Entity<'m, 'p>, &'e Location<'m>, Direction),
    NotOwnedEntity(&'e Entity<'m, 'p>, &'p Player),
}

impl<'p, 'm> Game<'p, 'm> {
    pub fn new<N: Into<String>>(name: N, round: u32, map: &'m Map) -> Game {
        Game {
            name: name.into(),
            round: round,
            map: map,
            entities: Entities::new(),
        }
    }

    pub fn view_for<'g>(&'g self, player: &'p Player) -> GameView<'p, 'm, 'g> {
        GameView {
            game: self,
            player: player,
        }
    }

    pub fn validate_move<'e>(
        &self,
        player: &'p Player,
        entity: &'e Entity<'m, 'p>,
        direction: Direction,
    ) -> Result<Location<'m>, GameRuleViolation<'p, 'm, 'e>> {
        let &Entity(_id, ref current_location, ref entity_type) = entity;

        match entity_type {
            &EntityType::Unit(owner, _) if ptr::eq(owner, player) => {
                if let Some(new_location) = current_location.in_direction(direction) {
                    if new_location.can_move_in() {
                        return Ok(new_location);
                    }
                }
                Err(GameRuleViolation::InvalidMove(
                    entity,
                    current_location,
                    direction,
                ))
            }
            _ => Err(GameRuleViolation::NotOwnedEntity(entity, player)),
        }
    }

    pub fn apply<A>(&mut self, desires: A)
    where
        A: Iterator<Item = Owned<'p, Desire>>,
    {
        for Owned(player, desire) in desires {
            println!("{}: {:?}", player.name, desire);
            match desire {
                Desire::Move(entity_id, direction) => {
                    let new_location =
                        if let Some(ref entity) = self.entities.get_by_entity_id(&entity_id) {
                            match self.validate_move(player, entity, direction) {
                                Err(err) => {
                                    println!("Invalid move: {:?}", err);
                                    continue;
                                }
                                Ok(new_location) => new_location,
                            }
                        } else {
                            // entity is gone
                            continue;
                        };

                    self.entities
                        .set_location_by_entity_id(&entity_id, new_location)
                        .expect("move was validated")
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Desire {
    Move(EntityID, Direction),
}

const GRID_INTERSECTION: &'static str = "+";
const GRID_HOR_LINE: &'static str = "---";
const GRID_VERT_LINE: &'static str = "|";
const GRID_EMPTY: &'static str = "   ";
const GRID_WALL: &'static str = "XXX";

const ENTITY_WORKER: &'static str = "W";
const ENTITY_LIGHT: &'static str = "L";
const ENTITY_HEAVY: &'static str = "H";
const ENTITY_BASE: &'static str = "@";
const ENTITY_BARRACS: &'static str = "B";
const ENTITY_RESOURCES: &'static str = "#";

impl<'p, 'm> Display for Game<'p, 'm> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fn write_grid_row_line(f: &mut fmt::Formatter, cels: usize) -> Result<(), fmt::Error> {
            write!(
                f,
                "{}",
                [GRID_INTERSECTION].iter().cycle().take(cels + 1).join(
                    GRID_HOR_LINE,
                )
            )
        }

        fn write_owned_entity(
            f: &mut fmt::Formatter,
            player: &Player,
            id: EntityID,
            symbol: &'static str,
        ) -> Result<(), fmt::Error> {
            write!(
                f,
                "{}",
                player.colour.paint(format!("{}{:02}", symbol, id.0))
            )
        }

        write_grid_row_line(f, self.map.width())?;
        for row in self.map.rows() {
            writeln!(f)?;
            for location in row {
                write!(f, "{}", GRID_VERT_LINE)?;
                match location.tile {
                    &Tile::Wall => write!(f, "{}", GRID_WALL)?,
                    &Tile::Empty => {
                        //TODO: merge join entities (ordered by coord, next() for peek().coord ==
                        // tile.coord
                        if let Some(&Entity(id, _, ref entity_type)) =
                            self.entities.get_by_location(&location)
                        {
                            match *entity_type {
                                EntityType::Unit(ref player, Unit::Worker) => {
                                    write_owned_entity(f, player, id, ENTITY_WORKER)?
                                }
                                EntityType::Unit(ref player, Unit::Light) => {
                                    write_owned_entity(f, player, id, ENTITY_LIGHT)?
                                }

                                EntityType::Unit(ref player, Unit::Heavy) => {
                                    write_owned_entity(f, player, id, ENTITY_HEAVY)?
                                }

                                EntityType::Building(ref player,
                                                     Building::Base(Resources(res))) => {
                                    write!(
                                        f,
                                        "{}",
                                        player.colour.paint(format!("{}{:02}", ENTITY_BASE, res))
                                    )?
                                }
                                EntityType::Building(ref player, Building::Barracks) => {
                                    write_owned_entity(f, player, id, ENTITY_BARRACS)?
                                }
                                EntityType::Resource(res) => {
                                    write!(f, "{}{:2}", ENTITY_RESOURCES, res)?
                                }
                            }
                        } else {
                            write!(f, "{}", GRID_EMPTY)?
                        }
                    }
                }
            }
            writeln!(f, "{}", GRID_VERT_LINE)?;
            write_grid_row_line(f, self.map.width())?;
        }

        Ok(())
    }
}
