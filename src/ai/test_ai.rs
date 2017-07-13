use std::ptr;

use game::{AI, EmptyPersistentState, GameView, Desire, Entity, EntityType, Unit, Direction};

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
                &Entity(_, ref location, EntityType::Unit(owner, Unit::Worker)) if ptr::eq(owner, view.player) => {
                    match location.in_direction(Direction::Right) {
                        Some(ref location) if location.can_move_in() => {
                            // just go right you entity!
                            actions.push(Desire::Move(*entity_id, Direction::Right));
                        }
                        _ => ()
                    }
                },
                _ => (),
            }
        }

        actions
    }
}
