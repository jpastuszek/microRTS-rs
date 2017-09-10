use pathfinding;
use game::{Direction, EntitiesIter, Entity, EntityID, Game, Location, Object, Player, Resource,
           Unit};
use std::ptr;
use std::hash::{Hash, Hasher};
use std::fmt;

#[derive(Debug)]
pub struct GameView<'p: 'g, 't: 'g, 'g> {
    game: &'g Game<'p, 't>,
    pub player: &'p Player,
}

impl<'p: 'g, 't: 'g, 'g> GameView<'p, 't, 'g> {
    pub fn new(game: &'g Game<'p, 't>, player: &'p Player) -> GameView<'p, 't, 'g> {
        GameView {
            game: game,
            player: player,
        }
    }

    pub fn navigator<'v>(&'v self, location: Location<'t>) -> Navigator<'p, 't, 'g, 'v> {
        let entity = self.game.get_entity_by_location(location);
        Navigator {
            game_view: self,
            location: location,
            entity: entity,
        }
    }

    pub fn entities(&self) -> EntitiesIter<'p, 't, 'g> {
        self.game.entities()
    }

    pub fn my_units<'v>(&'v self) -> MyUnitIter<'p, 't, 'g, 'v> {
        MyUnitIter {
            game_view: self,
            entities: self.entities(),
        }
    }

    pub fn resources<'v>(&'v self) -> ResourcesIter<'p, 't, 'g, 'v> {
        ResourcesIter {
            game_view: self,
            entities: self.entities(),
        }
    }
}

pub struct MyUnit<'p: 't, 't: 'g, 'g: 'v, 'v> {
    pub entity_id: EntityID,
    pub unit: &'g Unit,
    pub navigator: Navigator<'p, 't, 'g, 'v>,
}

pub struct MyUnitIter<'p: 't, 't: 'g, 'g: 'v, 'v> {
    game_view: &'v GameView<'p, 't, 'g>,
    entities: EntitiesIter<'p, 't, 'g>,
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> Iterator for MyUnitIter<'p, 't, 'g, 'v> {
    type Item = MyUnit<'p, 't, 'g, 'v>;

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

pub struct Resources<'p: 't, 't: 'g, 'g: 'v, 'v> {
    pub entity_id: EntityID,
    pub resource: &'g Resource,
    pub navigator: Navigator<'p, 't, 'g, 'v>,
}

pub struct ResourcesIter<'p: 't, 't: 'g, 'g: 'v, 'v> {
    game_view: &'v GameView<'p, 't, 'g>,
    entities: EntitiesIter<'p, 't, 'g>,
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> Iterator for ResourcesIter<'p, 't, 'g, 'v> {
    type Item = Resources<'p, 't, 'g, 'v>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((entity_id, entity)) = self.entities.next() {
                match entity {
                    &Entity {
                        location,
                        object: Object::Resources(ref resource),
                        ..
                    } => {
                        return Some(Resources {
                            entity_id: entity_id,
                            resource: resource,
                            navigator: self.game_view.navigator(location),
                        })
                    }
                    _ => continue,
                }
            } else {
                return None;
            }
        }
    }
}

//TODO: can I only have 'v?
pub struct Navigator<'p: 't, 't: 'g, 'g: 'v, 'v> {
    game_view: &'v GameView<'p, 't, 'g>,
    pub location: Location<'t>,
    pub entity: Option<&'g Entity<'t, 'p>>,
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> fmt::Debug for Navigator<'p, 't, 'g, 'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Navigator({}, {})[{:?}]",
            self.location.coordinates.0,
            self.location.coordinates.1,
            self.entity.map(|entity| entity.id)
        )
    }
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> PartialEq for Navigator<'p, 't, 'g, 'v> {
    fn eq(&self, other: &Navigator<'p, 't, 'g, 'v>) -> bool {
        self.location == other.location
    }
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> Eq for Navigator<'p, 't, 'g, 'v> {}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> Hash for Navigator<'p, 't, 'g, 'v> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> Clone for Navigator<'p, 't, 'g, 'v> {
    fn clone(&self) -> Self {
        Navigator {
            game_view: self.game_view,
            location: self.location.clone(),
            entity: self.entity,
        }
    }
}

impl<'p: 't, 't: 'g, 'g: 'v, 'v> Navigator<'p, 't, 'g, 'v> {
    pub fn in_direction(&self, direction: Direction) -> Option<Navigator<'p, 't, 'g, 'v>> {
        self.location
            .in_direction(direction)
            .map(|location| self.game_view.navigator(location))
    }

    pub fn find_path_dijkstra(
        &self,
        to: &Navigator<'p, 't, 'g, 'v>,
    ) -> Option<(Vec<Navigator<'p, 't, 'g, 'v>>, u64)> {
        let to_neighbour_locations = to.location
            .neighbours()
            .map(|(_direction, location)| location)
            .collect::<Vec<_>>();

        // TODO: if to is not walkable use neighbour location else use to directly
        pathfinding::dijkstra(
            self,
            |navigator| {
                navigator
                    .location
                    .neighbours()
                    .map(|(_direction, location)| {
                        (self.game_view.navigator(location), 1)
                    })
                    .filter(|&(ref target, _)| target.walkable() || target == to)
            },
            |navigator| to_neighbour_locations.contains(&navigator.location),
        )
    }

    pub fn direction_to(&self, to: &Navigator<'p, 't, 'g, 'v>) -> Option<Direction> {
        self.location.direction_to(to.location)
    }

    pub fn walkable(&self) -> bool {
        self.entity.is_none() && self.location.walkable()
    }
}
