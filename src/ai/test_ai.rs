use game::{AI, EmptyPersistentState, Desire, Unit};
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

        if let Some(resource) = view.resources().next() {
            for unit in view.my_units() {
                match unit.unit {
                    &Unit::Worker => {
                        //TODO: cleanup, remember path, select resource by distance
                        println!("{:?}", &unit.navigator);
                        if let Some((path, _cost)) = unit.navigator.find_path_dijkstra(&resource.navigator) {
                            println!("{:?}", &path);
                            if let Some(next_navigator) = path.iter().skip(1).next() {
                                if let Some(direction) = unit.navigator.direction_to(next_navigator) {
                                    desires.push(Desire::Move(unit.entity_id, direction));
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        desires
    }
}
