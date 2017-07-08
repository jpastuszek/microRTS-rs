mod game;
mod ai;

use game::{Player, AI, Owned, Game, EntityType, EntityID, Unit, Map, Coordinates};
use ai::idle_ai::IdleAI;
use ai::test_ai::TestAI;

fn main() {
    println!("Starting game");

    let rounds = 1;
    let cycles = 4;

    let map = Map::new(8, 8);

    let p1 = Player::new("Mario");
    let p2 = Player::new("Luigi");

    // not sure about state design
    let mut p1_state = p1.new_state::<IdleAI>();
    let mut p2_state = p2.new_state::<TestAI>();

    for round in 0..rounds {
        let mut game = Game::new("foo", round, &map);

        game.entities
            .place(
                game.map.location(Coordinates(0, 0)),
                EntityType::Resource(10),
            )
            .unwrap();
        game.entities
            .place(
                game.map.location(Coordinates(7, 7)),
                EntityType::Resource(10),
            )
            .unwrap();

        game.entities
            .place(
                game.map.location(Coordinates(2, 2)),
                EntityType::Unit(&p1, Unit::Worker),
            )
            .unwrap();
        game.entities
            .place(
                game.map.location(Coordinates(5, 5)),
                EntityType::Unit(&p2, Unit::Worker),
            )
            .unwrap();

        let mut p1_ai = p1.new_ai::<IdleAI>();
        let mut p2_ai = p2.new_ai::<TestAI>();

        for cycle in 0..cycles {
            let mut desires = Vec::new();


            {
                // Tag owner of desires
                let p1_desires = p1_ai
                    .update(&mut p1_state, game.view_for(&p1))
                    .into_iter()
                    .map(|d| Owned(&p1, d));
                let mut p2_desires = p2_ai
                    .update(&mut p2_state, game.view_for(&p2))
                    .into_iter()
                    .map(|d| Owned(&p2, d));

                // Interlave together
                // TODO: use itertools interleave()
                for p1_desire in p1_desires {
                    desires.push(p1_desire);
                    if let Some(p2_desire) = p2_desires.next() {
                        desires.push(p2_desire)
                    }
                }

                // remainign p2_actions
                for p2_desire in p2_desires {
                    desires.push(p2_desire);
                }
            }

            for desire in &desires {
                println!("[{}/{}] act: {:?}", cycle, round, desire);
            }

            game.apply(desires.into_iter());
            //println!("{:?}", game.entities.get_by_entity_id(&EntityID(3)));
            println!("{:?}", game);
        }
    }
}
