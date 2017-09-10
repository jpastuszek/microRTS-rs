use std::fmt::Display;
use std::fmt;
use std::ptr;
use itertools::Itertools;

use game::terrain::{Terrain, Direction, Coordinates, Location, Tile};
use game::entity::{Entity, Object, Iter as EntitiesIter, EntitiesError, Unit, Building,
                       Resource, Entities, EntityID};
use game::player::{Player, Owned};
use game_view::GameView;

#[derive(Debug)]
pub struct GameState<'p, 't> {
    name: String,
    round: u32,
    terrain: &'t Terrain,
    entities: Entities<'p, 't>,
}

//TODO: Error trait
// This type cannot keep references to Game or Entity so it can be passed back to AI causing the
// violation
#[derive(Debug)]
pub enum GameRuleViolation<'p, 't> {
    InvalidMove(EntityID, Direction, InvalidMove<'t>),
    EntityNotOwned(EntityID, &'p Player),
    EntityDoesNotExist(EntityID),
}

#[derive(Debug)]
pub enum InvalidMove<'t> {
    NotWalkable(Location<'t>),
    LocationAlreadyTaken(Location<'t>, EntityID),
    Immovable,
    OutOfMap,
}

impl<'p, 't> GameState<'p, 't> {
    pub fn view_for<'s>(&'s self, player: &'p Player) -> GameView<'p, 't, 's> {
        GameView::new(self, player)
    }

    pub fn entities<'s>(&'s self) -> EntitiesIter<'p, 't, 's> {
        self.entities.iter()
    }

    pub fn get_entity_by_location<'s> (&'s self, location: Location<'t>) -> Option<&'s Entity<'t, 'p>> {
        self.entities.get_by_location(location)
    }

    fn move_entity(
        &mut self,
        player: &'p Player,
        entity_id: EntityID,
        direction: Direction,
    ) -> Result<(), GameRuleViolation<'p, 't>> {
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
                    EntitiesError::LocationAlreadyOccupied(new_location, by_entity_id) => {
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
pub enum GameStateBuilderError<'t> {
    OutOfMap(Coordinates),
    EntityPlaceError(EntitiesError<'t>)
}

#[derive(Debug)]
pub struct GameStateBuilder<'p, 't> {
    name: String,
    terrain: &'t Terrain,
    entities: Entities<'p, 't>,
}

impl<'p, 't> GameStateBuilder<'p, 't> {
    pub fn new<N: Into<String>>(name: N, terrain: &'t Terrain) -> GameStateBuilder<'p, 't> {
        GameStateBuilder {
            name: name.into(),
            terrain: terrain,
            entities: Entities::new(),
        }
    }

    pub fn place(&mut self, coordinates: Coordinates, object: Object<'p>) -> Result<&mut GameStateBuilder<'p, 't>, GameStateBuilderError<'t>> {
        self.terrain.location(coordinates)
            .ok_or_else(|| GameStateBuilderError::OutOfMap(coordinates))
            .and_then(|location| self.entities.place(location, object)
                      .map_err(|place_error| GameStateBuilderError::EntityPlaceError(place_error)))
            .map(|_| self)
    }

    pub fn build_for_round(&self, round: u32) -> GameState<'p, 't> {
        GameState {
            name: self.name.clone(),
            round: round,
            terrain: self.terrain.clone(),
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

impl<'p, 't> Display for GameState<'p, 't> {
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

        write_grid_row_line(f, self.terrain.width())?;
        for row in self.terrain.rows() {
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
            write_grid_row_line(f, self.terrain.width())?;
        }

        Ok(())
    }
}
