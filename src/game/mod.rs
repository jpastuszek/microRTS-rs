mod map;
mod entity;
mod player;

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
    NotOwnedEntity(&'e Entity<'m, 'p>, &'p Player)
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
            player: player
        }
    }

    pub fn validate_move<'e>(&self, player: &'p Player, entity: &'e Entity<'m, 'p>, direction: Direction) -> Result<Location<'m>, GameRuleViolation<'p, 'm, 'e>> {
        let &Entity(Location(ref coordinates, _), ref entity_type) = entity;
        match entity_type {
            &EntityType::Unit(owner, _) if owner == player => {
                let new_location = self.map.location(coordinates.in_direction(direction));
                match new_location.1 {
                    &Tile::Plain => Ok(new_location),
                    _ => Err(GameRuleViolation::BadMoveLocation(entity, new_location))
                }
            }
            _ => Err(GameRuleViolation::NotOwnedEntity(entity, player))
        }
    }

    pub fn apply<A>(&mut self, desires: A) where A: Iterator<Item=Owned<'p, Desire>> {
        for Owned(player, desire) in desires {
            println!("{}: {:?}", player.name, desire);
            match desire {
                Desire::Move(entity_id, direction) => {
                    let new_location = if let Some(ref entity) = self.entities.get_by_entity_id(&entity_id) {
                        match self.validate_move(player, entity, direction) {
                            Err(err) => {
                                println!("Invalid move: {:?}", err);
                                continue
                            },
                            Ok(new_location) => new_location
                        }
                    } else {
                        // entity is gone
                        continue
                    };

                    self.entities.set_location_by_entity_id(&entity_id, new_location).expect("move was validated")
                }
            }
        }
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
