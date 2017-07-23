use game::{AI, EmptyPersistentState, Desire, Unit, Direction};
use game_view::GameView;

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

        for unit in view.my_units() {
            match unit.unit {
                // Desires cannot hold references to anything
                // inside Game or we can't modify it later on
                &Unit::Worker => {
                    match unit.navigator.in_direction(Direction::Right) {
                        Some(ref navigator) if navigator.walkable() => {
                            // just go right you entity!
                            actions.push(Desire::Move(unit.entity_id, Direction::Right));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        actions
    }
}
