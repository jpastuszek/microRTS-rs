mod map;
mod entity;
mod player;

use std::fmt::Display;
use std::fmt;
use std::ptr;
use itertools::Itertools;

// Flat structure for AI
pub use game::map::{Map, Dimension, Direction, Coordinates, Location, Tile};
pub use game::entity::{Entity, Object, Iter as EntitiesIter, EntitiesError, Unit, Building,
                       Resource, Entities, EntityID};
pub use game::player::{Player, Colour, AI, EmptyPersistentState, Owned};
use game_view::GameView;

#[derive(Debug)]
pub struct Game<'p, 'm> {
    name: String,
    round: u32,
    map: &'m Map,
    entities: Entities<'p, 'm>,
}

//TODO: Error trait
// This type cannot keep references to Game or Entity so it can be passed back to AI causing the
// violation
#[derive(Debug)]
pub enum GameRuleViolation<'p, 'm> {
    InvalidMove(EntityID, Direction, InvalidMove<'m>),
    EntityNotOwned(EntityID, &'p Player),
    EntityDoesNotExist(EntityID),
}

#[derive(Debug)]
pub enum InvalidMove<'m> {
    NotWalkable(Location<'m>),
    LocationAlreadyTaken(Location<'m>, EntityID),
    Immovable,
    OutOfMap,
}

impl<'p, 'm> Game<'p, 'm> {
    pub fn new(name: String, round: u32, map: &'m Map) -> Game {
        Game {
            name: name,
            round: round,
            map: map,
            entities: Entities::new(),
        }
    }

    pub fn view_for<'g>(&'g self, player: &'p Player) -> GameView<'p, 'm, 'g> {
        GameView::new(self, player)
    }

    pub fn entities<'g>(&'g self) -> EntitiesIter<'p, 'm, 'g> {
        self.entities.iter()
    }

    pub fn get_entity_by_location<'g> (&'g self, location: Location<'m>) -> Option<&'g Entity<'m, 'p>> {
        self.entities.get_by_location(location)
    }

    fn move_entity(
        &mut self,
        player: &'p Player,
        entity_id: EntityID,
        direction: Direction,
    ) -> Result<(), GameRuleViolation<'p, 'm>> {
        if let Some(ref mut entity_mutator) = self.entities.get_mutator(entity_id) {
            let current_location = match entity_mutator.entity.object {
                Object::Building(..) |
                Object::Resources(..) => {
                    return Err(GameRuleViolation::InvalidMove(
                        entity_mutator.entity.id,
                        direction,
                        InvalidMove::Immovable,
                    ))
                }
                Object::Unit(owner, _) => {
                    if !ptr::eq(owner, player) {
                        return Err(GameRuleViolation::EntityNotOwned(
                            entity_mutator.entity.id,
                            player,
                        ));
                    }
                    entity_mutator.entity.location
                }
            };

            let new_location = match current_location.in_direction(direction) {
                None => {
                    return Err(GameRuleViolation::InvalidMove(
                        entity_mutator.entity.id,
                        direction,
                        InvalidMove::OutOfMap,
                    ))
                }
                Some(new_location) => new_location,
            };

            entity_mutator.set_location(new_location).map_err(
                |err| match err {
                    EntitiesError::LocationNotWalkable(new_location) => {
                        GameRuleViolation::InvalidMove(
                            entity_id,
                            direction,
                            InvalidMove::NotWalkable(new_location),
                        )
                    }
                    EntitiesError::LocationAlreadyTaken(new_location, by_entity_id) => {
                        GameRuleViolation::InvalidMove(
                            entity_id,
                            direction,
                            InvalidMove::LocationAlreadyTaken(new_location, by_entity_id),
                        )
                    }
                },
            )
        } else {
            return Err(GameRuleViolation::EntityDoesNotExist(entity_id));
        }
    }

    pub fn apply<A>(&mut self, desires: A)
    where
        A: Iterator<Item = Owned<'p, Order>>,
    {
        for Owned(player, desire) in desires {
            println!("{}: {:?}", player.name, desire);
            match desire {
                Order::Move(entity_id, direction) => {
                    self.move_entity(player, entity_id, direction).expect(
                        "TODO: collect rule violations and pass to AI",
                    )
                }
            }
        }
    }
}

//TODO: Error
#[derive(Debug)]
pub enum GameBuilderError<'m> {
    OutOfMap(Coordinates),
    EntityPlaceError(EntitiesError<'m>)
}

#[derive(Debug)]
pub struct GameBuilder<'p, 'm> {
    name: String,
    map: &'m Map,
    entities: Entities<'p, 'm>,
}

impl<'p, 'm> GameBuilder<'p, 'm> {
    pub fn new<N: Into<String>>(name: N, map: &'m Map) -> GameBuilder<'p, 'm> {
        GameBuilder {
            name: name.into(),
            map: map,
            entities: Entities::new(),
        }
    }

    pub fn place(&mut self, coordinates: Coordinates, object: Object<'p>) -> Result<&mut GameBuilder<'p, 'm>, GameBuilderError<'m>> {
        self.map.location(coordinates)
            .ok_or_else(|| GameBuilderError::OutOfMap(coordinates))
            .and_then(|location| self.entities.place(location, object)
                      .map_err(|place_error| GameBuilderError::EntityPlaceError(place_error)))
            .map(|_| self)
    }

    pub fn build(&self, round: u32) -> Game<'p, 'm> {
        Game {
            name: self.name.clone(),
            round: round,
            map: self.map.clone(),
            entities: self.entities.clone(),
        }
    }
}

#[derive(Debug)]
pub enum Order {
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
                        if let Some(ref entity) = self.entities.get_by_location(location) {
                            match entity.object {
                                Object::Unit(ref player, Unit::Worker) => {
                                    write_owned_entity(f, player, entity.id, ENTITY_WORKER)?
                                }
                                Object::Unit(ref player, Unit::Light) => {
                                    write_owned_entity(f, player, entity.id, ENTITY_LIGHT)?
                                }

                                Object::Unit(ref player, Unit::Heavy) => {
                                    write_owned_entity(f, player, entity.id, ENTITY_HEAVY)?
                                }

                                Object::Building(ref player, Building::Base(Resource(res))) => {
                                    write!(
                                        f,
                                        "{}",
                                        player.colour.paint(format!("{}{:02}", ENTITY_BASE, res))
                                    )?
                                }
                                Object::Building(ref player, Building::Barracks) => {
                                    write_owned_entity(f, player, entity.id, ENTITY_BARRACS)?
                                }
                                Object::Resources(Resource(res)) => {
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
