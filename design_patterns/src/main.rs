use state::{Item, Resolver};

mod arc_rw_lock_wrapper;
mod state;
mod looper;

fn main() {
    let name = String::from("test");
    let mut resolver = Resolver::default();

    resolver.add(Item { name: &name });
}
