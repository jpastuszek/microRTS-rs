use game::{AI, EmptyPersistentState, Order, Unit};
use game_view::GameView;

#[derive(Default)]
pub struct TestAI;

impl AI for TestAI {
    type PersistentState = EmptyPersistentState;
    fn update<'p: 'g, 's: 'g, 't: 'g, 'g>(
        &mut self,
        _sate: &'s mut Self::PersistentState,
        view: GameView<'p, 't, 'g>,
    ) -> Vec<Order> {
        let mut desires = Vec::new();

        //TODO: remember path calculation
        for unit in view.my_units() {
            match unit.unit {
                &Unit::Worker => {
                    let mut resources_paths = view.resources()
                        .filter_map(|resource| unit.navigator.find_path_dijkstra(&resource.navigator))
                        .collect::<Vec<_>>();

                    resources_paths.sort_by_key(|&(_, cost)| cost);

                    if let Some(&(ref path, _cost)) = resources_paths.first() {
                        path.iter().skip(1).next()
                            .and_then(|next_navigator| unit.navigator.direction_to(next_navigator))
                            .map(|direction| desires.push(Order::Move(unit.entity_id, direction)));
                    }
                }
                _ => (),
            }
        }

        desires
    }
}
