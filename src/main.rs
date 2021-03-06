extern crate micro_rts;
extern crate itertools;

use itertools::interleave;

use micro_rts::game::{Building, Colour, Coordinates, GameStateBuilder, TerrainBuilder, Object, Owned, Player,
           Resource, Unit, AI};
use micro_rts::ai::idle_ai::IdleAI;
use micro_rts::ai::test_ai::TestAI;

fn main() {
    println!("Starting game");

    let rounds = 1;
    let cycles = 5;

    let terrain = TerrainBuilder::terrain_8x8_wall1();

    let p1 = Player::new("Mario", Colour::Red);
    let p2 = Player::new("Luigi", Colour::Green);

    // not sure about state design
    let mut p1_state = p1.new_state::<IdleAI>();
    let mut p2_state = p2.new_state::<TestAI>();

    let mut game_builder = GameStateBuilder::new("foo", &terrain);

    game_builder
        .place(Coordinates(0, 0), Object::Resources(Resource(10)))
        .unwrap()
        .place(Coordinates(7, 7), Object::Resources(Resource(10)))
        .unwrap()
        .place(
            Coordinates(2, 1),
            Object::Building(&p1, Building::Base(Resource(10))),
        )
        .unwrap()
        .place(
            Coordinates(5, 6),
            Object::Building(&p2, Building::Base(Resource(10))),
        )
        .unwrap()
        .place(Coordinates(2, 2), Object::Unit(&p1, Unit::Worker))
        .unwrap()
        .place(Coordinates(5, 5), Object::Unit(&p2, Unit::Worker))
        .unwrap();

    for round in 0..rounds {
        let mut game = game_builder.build_for_round(round);

        println!("{}", game);

        let mut p1_ai = p1.new_ai::<IdleAI>();
        let mut p2_ai = p2.new_ai::<TestAI>();

        for cycle in 0..cycles {
            println!();
            println!("Cycle: {}", cycle);

            // Tag owner of desires
            let p1_desires = p1_ai
                .update(&mut p1_state, game.view_for(&p1))
                .into_iter()
                .map(|d| Owned(&p1, d));
            let p2_desires = p2_ai
                .update(&mut p2_state, game.view_for(&p2))
                .into_iter()
                .map(|d| Owned(&p2, d));

            game.apply(interleave(p1_desires, p2_desires));

            println!("{}", game);
        }
    }
}
