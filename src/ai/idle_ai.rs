use game::{AI, EmptyPersistentState, GameView, Desire};

#[derive(Default)]
pub struct IdleAI;

impl AI for IdleAI {
    type PersistentState = EmptyPersistentState;
    fn update<'p: 'g, 's: 'g, 'm: 'g, 'g>(
        &mut self,
        _sate: &'s mut Self::PersistentState,
        _view: GameView<'p, 'm, 'g>,
    ) -> Vec<Desire> {
        // no desires!
        Vec::new()
    }
}
