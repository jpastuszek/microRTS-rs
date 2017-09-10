use pathfinding;
use game::{GameState, Entity, EntityID, Object, Unit, EntitiesIter, Player, Location, Direction, Resource};
use std::ptr;
use std::hash::{Hash, Hasher};
use std::fmt;

#[derive(Debug)]
pub struct GameView<'p: 's, 'm: 's, 's> {
    game: &'s GameState<'p, 'm>,
    pub player: &'p Player,
}

impl<'p: 's, 'm: 's, 's> GameView<'p, 'm, 's> {
    pub fn new(game: &'s GameState<'p, 'm>, player: &'p Player) -> GameView<'p, 'm, 's> {
        GameView {
            game: game,
            player: player,
        }
    }

    pub fn navigator<'v>(&'v self, location: Location<'m>) -> Navigator<'p, 'm, 's, 'v> {
        let entity = self.game.get_entity_by_location(location);
        Navigator {
            game_view: self,
            location: location,
            entity: entity,
        }
    }

    pub fn entities(&self) -> EntitiesIter<'p, 'm, 's> {
        self.game.entities()
    }

    pub fn my_units<'v>(&'v self) -> MyUnitIter<'p, 'm, 's, 'v> {
        MyUnitIter {
            game_view: self,
            entities: self.entities(),
        }
    }

    pub fn resources<'v>(&'v self) -> ResourcesIter<'p, 'm, 's, 'v> {
        ResourcesIter {
            game_view: self,
            entities: self.entities(),
        }
    }
}

pub struct MyUnit<'p: 'm, 'm: 's, 's: 'v, 'v> {
    pub entity_id: EntityID,
    pub unit: &'s Unit,
    pub navigator: Navigator<'p, 'm, 's, 'v>,
}

pub struct MyUnitIter<'p: 'm, 'm: 's, 's: 'v, 'v> {
    game_view: &'v GameView<'p, 'm, 's>,
    entities: EntitiesIter<'p, 'm, 's>,
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> Iterator for MyUnitIter<'p, 'm, 's, 'v> {
    type Item = MyUnit<'p, 'm, 's, 'v>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((entity_id, entity)) = self.entities.next() {
                match entity {
                    &Entity { location, object: Object::Unit(owner, ref unit), .. }
                        //TODO: should that be impl Eq for Player?
                        if ptr::eq(owner, self.game_view.player) => {
                            return Some(MyUnit {
                                entity_id: entity_id,
                                unit: unit,
                                navigator: self.game_view.navigator(location)
                            })
                        }
                    _ => continue
                }
            } else {
                return None;
            }
        }
    }
}

pub struct Resources<'p: 'm, 'm: 's, 's: 'v, 'v> {
    pub entity_id: EntityID,
    pub resource: &'s Resource,
    pub navigator: Navigator<'p, 'm, 's, 'v>,
}

pub struct ResourcesIter<'p: 'm, 'm: 's, 's: 'v, 'v> {
    game_view: &'v GameView<'p, 'm, 's>,
    entities: EntitiesIter<'p, 'm, 's>,
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> Iterator for ResourcesIter<'p, 'm, 's, 'v> {
    type Item = Resources<'p, 'm, 's, 'v>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((entity_id, entity)) = self.entities.next() {
                match entity {
                    &Entity { location, object: Object::Resources(ref resource), .. } => {
                        return Some(Resources {
                            entity_id: entity_id,
                            resource: resource,
                            navigator: self.game_view.navigator(location)
                        })
                    }
                    _ => continue
                }
            } else {
                return None;
            }
        }
    }
}

//TODO: can I only have 'v?
pub struct Navigator<'p: 'm, 'm: 's, 's: 'v, 'v> {
    game_view: &'v GameView<'p, 'm, 's>,
    pub location: Location<'m>,
    pub entity: Option<&'s Entity<'m, 'p>>,
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> fmt::Debug for Navigator<'p, 'm, 's, 'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Navigator({}, {})[{:?}]", self.location.coordinates.0, self.location.coordinates.1, self.entity.map(|entity| entity.id))
    }
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> PartialEq for Navigator<'p, 'm, 's, 'v> {
    fn eq(&self, other: &Navigator<'p, 'm, 's, 'v>) -> bool {
        self.location == other.location
    }
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> Eq for Navigator<'p, 'm, 's, 'v> {}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> Hash for Navigator<'p, 'm, 's, 'v> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> Clone for Navigator<'p, 'm, 's, 'v> {
    fn clone(&self) -> Self {
        Navigator {
            game_view: self.game_view,
            location: self.location.clone(),
            entity: self.entity
        }
    }
}

impl<'p: 'm, 'm: 's, 's: 'v, 'v> Navigator<'p, 'm, 's, 'v> {
    pub fn in_direction(&self, direction: Direction) -> Option<Navigator<'p, 'm, 's, 'v>> {
        self.location.in_direction(direction).map(|location| {
            self.game_view.navigator(location)
        })
    }

    pub fn find_path_dijkstra(&self, to: &Navigator<'p, 'm, 's, 'v>) -> Option<(Vec<Navigator<'p, 'm, 's, 'v>>, u64)> {
        let to_neighbour_locations = to.location.neighbours().map(|(_direction, location)| location).collect::<Vec<_>>();

        // TODO: if to is not walkable use neighbour location else use to directly
        pathfinding::dijkstra(
            self,
            |navigator| navigator.location.neighbours()
                .map(|(_direction, location)| (self.game_view.navigator(location), 1))
                .filter(|&(ref target, _)| target.walkable() || target == to),
            |navigator| to_neighbour_locations.contains(&navigator.location)
        )
    }

    pub fn direction_to(&self, to: &Navigator<'p, 'm, 's, 'v>) -> Option<Direction> {
        self.location.direction_to(to.location)
    }

    pub fn walkable(&self) -> bool {
        self.entity.is_none() && self.location.walkable()
    }
}