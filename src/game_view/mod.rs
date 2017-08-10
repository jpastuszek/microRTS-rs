use game::{Game, Entity, EntityID, Object, Unit, EntitiesIter, Player, Location, Direction};
use std::ptr;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct GameView<'p: 'g, 'm: 'g, 'g> {
    game: &'g Game<'p, 'm>,
    pub player: &'p Player,
}
// TODO: GameView should build NavigationMap which includes Map tiles and Entities and can be
// navigated with Navigators (like Location but over NavigaionMap and with path finding stuff)

/*
pub struct NavigationMap<'p: 'm, 'm: 'e, 'e> {
    map: &'m Map,
    entities: &'e Entities<'p, 'm>,
}

impl<'p: 'm, 'm: 'e, 'e> NavigationMap<'p, 'm, 'e> {
*/

impl<'p: 'g, 'm: 'g, 'g> GameView<'p, 'm, 'g> {
    pub fn new(game: &'g Game<'p, 'm>, player: &'p Player) -> GameView<'p, 'm, 'g> {
        GameView {
            game: game,
            player: player,
        }
    }

    pub fn navigator<'v>(&'v self, location: Location<'m>) -> Navigator<'p, 'm, 'g, 'v> {
        let entity = self.game.get_entity_by_location(location);
        Navigator {
            game_view: self,
            location: location,
            entity: entity,
        }
    }

    pub fn entities(&self) -> EntitiesIter<'p, 'm, 'g> {
        self.game.entities()
    }

    pub fn my_units<'v>(&'v self) -> MyUnits<'p, 'm, 'g, 'v> {
        MyUnits {
            game_view: self,
            entities: self.entities(),
        }
    }
}

pub struct MyUnit<'p: 'm, 'm: 'g, 'g: 'v, 'v> {
    pub entity_id: EntityID,
    pub unit: &'g Unit,
    pub navigator: Navigator<'p, 'm, 'g, 'v>,
}

pub struct MyUnits<'p: 'm, 'm: 'g, 'g: 'v, 'v> {
    game_view: &'v GameView<'p, 'm, 'g>,
    entities: EntitiesIter<'p, 'm, 'g>,
}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> Iterator for MyUnits<'p, 'm, 'g, 'v> {
    type Item = MyUnit<'p, 'm, 'g, 'v>;

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

pub struct Navigator<'p: 'm, 'm: 'g, 'g: 'v, 'v> {
    game_view: &'v GameView<'p, 'm, 'g>,
    pub location: Location<'m>,
    pub entity: Option<&'g Entity<'m, 'p>>,
}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> PartialEq for Navigator<'p, 'm, 'g, 'v> {
    fn eq(&self, other: &Navigator<'p, 'm, 'g, 'v>) -> bool {
        self.location == other.location
    }
}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> Eq for Navigator<'p, 'm, 'g, 'v> {}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> Hash for Navigator<'p, 'm, 'g, 'v> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> Clone for Navigator<'p, 'm, 'g, 'v> {
    fn clone(&self) -> Self {
        Navigator {
            game_view: self.game_view,
            location: self.location.clone(),
            entity: self.entity
        }
    }
}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> Navigator<'p, 'm, 'g, 'v> {
    pub fn in_direction(&self, direction: Direction) -> Option<Navigator<'p, 'm, 'g, 'v>> {
        self.location.in_direction(direction).map(|location| {
            self.game_view.navigator(location)
        })
    }

    pub fn path_dijkstra(&self, to: Navigator<'p, 'm, 'g, 'v>) -> Option<(Vec<Navigator<'p, 'm, 'g, 'v>>, u64)> {
        use pathfinding::dijkstra;
        dijkstra(
            self,
            |navigator| navigator.location.neighbours()
                .map(|(_direction, location)| (self.game_view.navigator(location), 1)),
            |navigator| navigator.location == to.location
        )
    }

    pub fn walkable(&self) -> bool {
        self.entity.is_none() && self.location.walkable()
    }
}
