use game::{Game, Entities, Entity, EntitiesIter, Player, Map, Location, Direction};

//TODO: all pub exprots for AI should be here

#[derive(Debug)]
pub struct GameView<'p: 'g, 'm: 'g, 'g> {
    //TODO: make private
    pub game: &'g Game<'p, 'm>,
    pub player: &'p Player,
}
// TODO: AI should only be able to call methods on GameView - make game private
// TODO: GameView should build NavigationMap which includes Map tiles and Entities and can be
// navigated with Navigators (like Location but over NavigaionMap and with path finding stuff)
// TODO: move to GameView
//

/*
pub struct NavigationMap<'p: 'm, 'm: 'e, 'e> {
    map: &'m Map,
    entities: &'e Entities<'p, 'm>,
}

impl<'p: 'm, 'm: 'e, 'e> NavigationMap<'p, 'm, 'e> {
*/

impl<'p: 'g, 'm: 'g, 'g> GameView<'p, 'm, 'g> {
    pub fn navigator<'v>(&'v self, location: Location<'m>) -> Navigator<'p, 'm, 'g, 'v> {
        // TODO: index entities in map like matrix for quick access
        let entity = self.game.entities.get_by_location(&location);
        Navigator {
            game_view: self,
            location: location,
            entity: entity
        }
    }

    pub fn entities<'e>(&'e self) -> EntitiesIter<'p, 'm, 'e> {
        self.game.entities.iter()
    }

    // TODO: my_units that will filter by player
}

pub struct Navigator<'p: 'm, 'm: 'g, 'g: 'v, 'v> {
    game_view: &'v GameView<'p, 'm, 'g>,
    pub location: Location<'m>,
    pub entity: Option<&'g Entity<'m, 'p>>
}

impl<'p: 'm, 'm: 'g, 'g: 'v, 'v> Navigator<'p, 'm, 'g, 'v> {
    pub fn in_direction(&self, direction: Direction) -> Option<Navigator<'p, 'm, 'g, 'v>> {
        self.location.in_direction(direction).map(
            |location| {
                self.game_view.navigator(location)
            },
        )
    }
}
