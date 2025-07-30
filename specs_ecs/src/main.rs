use specs::prelude::*;

mod components;
// mod events;
// mod systems;
// mod bundles;
// mod spawner;
// mod setup;

// use systems::tick::Tick;

fn main() {
    let mut world = World::new();
    // setup::register_components(&mut world);
    // setup::create_entities(&mut world);

    // let mut dispatcher = setup::create_dispatcher();
    // dispatcher.setup(&mut world);

    // for _ in 0..20 {
    //     dispatcher.dispatch(&world);
    //     world.maintain();
    //     world.write_resource::<Tick>().0 += 1;
    // }
}
