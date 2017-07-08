mod map;
mod entity;
mod player;

use std::fmt::Display;
use std::fmt;
use itertools::Itertools;

pub use self::map::{Map, Direction, Coordinates, Location, Tile};
pub use self::entity::{Entity, EntityType, Unit, Building, Entities, EntityID};
pub use self::player::{Player, AI, EmptyPersistentState, Owned};

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
    BadMoveLocation(&'e Entity<'m, 'p>, Location<'m>),
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
        let &Entity(_id, Location(ref coordinates, _), ref entity_type) = entity;
        match entity_type {
            &EntityType::Unit(owner, _) if owner == player => {
                let new_location = self.map.location(coordinates.in_direction(direction));
                match new_location.1 {
                    &Tile::Plain => Ok(new_location),
                    _ => Err(GameRuleViolation::BadMoveLocation(entity, new_location)),
                }
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

const GRID_INTERSECTION: &'static str = "+";
const GRID_HOR_LINE: &'static str = "---";
const GRID_VERT_LINE: &'static str = "|";
const GRID_EMPTY: &'static str = "   ";
const GRID_WALL: &'static str = "XXX";
const ENTITY_WORKER: &'static str = "W";

impl<'p, 'm> Display for Game<'p, 'm> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fn write_grid_row_line(f: &mut fmt::Formatter, cels: usize) -> Result<(), fmt::Error> {
            writeln!(f, "{}", [GRID_INTERSECTION].iter().cycle().take(cels + 1).join(GRID_HOR_LINE))
        }

        write_grid_row_line(f, self.map.width())?;
        for row in self.map.rows() {
            //writeln!(f, "{}", [GRID_VERT_LINE].iter().cycle().take(self.map.width() + 1).join(GRID_EMPTY))?;
            for location in row {
                write!(f, "{}", GRID_VERT_LINE)?;
                match location {
                    Location(_, &Tile::Void) => write!(f, "{}", GRID_EMPTY)?,
                    Location(_, &Tile::Wall) => write!(f, "{}", GRID_WALL)?,
                    Location(_, &Tile::Plain) => {
                        //TODO: merge join entities (ordered by coord, next() for peek().coord ==
                        // tile.coord
                        if let Some(&Entity(id, _, ref _entity_type)) = self.entities.get_by_location(&location) {
                            // TODO: render different types
                            write!(f, "{}{:02}", ENTITY_WORKER, id.0)?
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

#[derive(Debug)]
pub struct GameView<'p: 'g, 'm: 'g, 'g> {
    pub game: &'g Game<'p, 'm>,
    pub player: &'p Player,
}
// TODO: AI should only be able to call methods on GameView - make game private

#[derive(Debug)]
pub enum Desire {
    Move(EntityID, Direction),
}
