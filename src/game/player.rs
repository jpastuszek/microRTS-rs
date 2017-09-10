pub use ansi_term::Colour;

use game::game_state::Order;
use game_view::GameView;

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub colour: Colour,
}

impl Player {
    pub fn new<N: Into<String>>(name: N, colour: Colour) -> Player {
        Player {
            name: name.into(),
            colour: colour,
        }
    }

    // called per game round
    pub fn new_ai<A: AI>(&self) -> A {
        A::default()
    }

    // called once per game
    pub fn new_state<A: AI>(&self) -> A::PersistentState {
        A::PersistentState::default()
    }
}

#[derive(Debug)]
pub struct Owned<'p, T>(pub &'p Player, pub T);

pub trait AI: Default {
    // Struct to keep data between game rounds
    type PersistentState: Default;

    // Returning Vec so that we can make sure that AI run is finished when this function returns
    // Should take Events as input (slice?) which are effect of applying actions and can be used
    // to keep track of changes
    fn update<'p: 's, 's: 'gs, 't: 'gs, 'gs>(
        &mut self,
        sate: &'s mut Self::PersistentState,
        view: GameView<'p, 't, 'gs>,
    ) -> Vec<Order>;
}

#[derive(Default)]
pub struct EmptyPersistentState;
