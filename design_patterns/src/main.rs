use resolver::{Item, Resolver};

mod arc_rw_lock_wrapper;
mod resolver;

fn main() {
    let name = String::from("test");
    let mut resolver = Resolver::default();

    resolver.add(Item { name: &name });
}
