use std::collections::HashMap;

#[derive(Debug)]
pub struct Item<'a> {
    name: &'a str
}

#[derive(Default, Debug)]
pub struct Resolver<'a> {
    entities: HashMap<&'a str, Item<'a>>
}

impl<'a> Resolver<'a> {
    pub fn add(&mut self, item: Item<'a>) {
        self.entities.insert(&item.name, item);
    }
}

pub struct Service<'a> {
    resolver: Resolver<'a>
}

impl<'a> Service<'a> {
    pub fn run(&mut self) {
        let item = Item {
            name: "test"
        };

        self.resolver.add(item);
    }
}

fn main() {
    let name = String::from("test");
    let mut resolver = Resolver::default();

    resolver.add(Item { name: &name });
}
