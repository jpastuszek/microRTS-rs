use game::{AI, EmptyPersistentState, GameView, Desire, Entity, EntityType, Unit, Location,
           Direction, Tile};

#[derive(Default)]
pub struct TestAI;

impl AI for TestAI {
    type PersistentState = EmptyPersistentState;
    fn update<'p: 'g, 's: 'g, 'm: 'g, 'g>(
        &mut self,
        _sate: &'s mut Self::PersistentState,
        view: GameView<'p, 'm, 'g>,
    ) -> Vec<Desire> {
        let mut actions = Vec::new();

        for (entity_id, entity) in &view.game.entities {
            match entity {
                // Desires cannot hold references to anything
                // inside Game or we can't modify it later on
                &Entity(Location(ref coordinates, _), EntityType::Unit(owner, Unit::Worker))
                    if owner == view.player => {
                    if let Location(_pos, &Tile::Plain) =
                        view.game.map.location(
                            coordinates.in_direction(Direction::Right),
                        )
                    {
                        // just go right you entity!
                        actions.push(Desire::Move(*entity_id, Direction::Right));
                    }
                }
                _ => (),
            }
        }

        actions
    }
}
