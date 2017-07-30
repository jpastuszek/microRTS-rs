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
        let mut desires = Vec::new();

        for unit in view.my_units() {
            match unit.unit {
                &Unit::Worker => {
                    match unit.navigator.in_direction(Direction::Right) {
                        Some(ref navigator) if navigator.walkable() => {
                            // just go right you entity!
                            desires.push(Desire::Move(unit.entity_id, Direction::Right));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        desires
    }
}
