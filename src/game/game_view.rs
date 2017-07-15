use game::Game;
use game::Player;

#[derive(Debug)]
pub struct GameView<'p: 'g, 'm: 'g, 'g> {
    pub game: &'g Game<'p, 'm>,
    pub player: &'p Player,
}
// TODO: AI should only be able to call methods on GameView - make game private
// TODO: GameView should build NavigationMap which includes Map tiles and Entities and can be
// navigated with Navigators (like Location but over NavigaionMap and with path finding stuff)
// TODO: move to GameView
//
