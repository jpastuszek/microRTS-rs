use game::{AI, EmptyPersistentState, Order};
use game_view::GameView;

#[derive(Default)]
pub struct IdleAI;

impl AI for IdleAI {
    type PersistentState = EmptyPersistentState;
    fn update<'p: 'g, 's: 'g, 'm: 'g, 'g>(
        &mut self,
        _sate: &'s mut Self::PersistentState,
        _view: GameView<'p, 'm, 'g>,
    ) -> Vec<Order> {
        // no desires!
        Vec::new()
    }
}
