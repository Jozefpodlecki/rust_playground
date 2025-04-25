use std::{collections::HashMap, hash::Hash, collections::hash_map::Entry};

pub struct Item {
    name: String,
    value: u64,
}

pub struct State {
    items: HashMap<u64, Item>,
}

impl State {
    pub fn new() -> Self {
        let mut items = HashMap::new();
        items.insert(1, Item { name: "1".to_string(), value: 1 });
        Self { items }
    }

    pub fn get_item(&mut self, id: u64) -> &mut Item {
        match self.items.entry(id) {
            Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            Entry::Vacant(vacant_entry) => vacant_entry.insert(Item { name: id.to_string(), value: 0 }),
        }
    }

    pub fn further_modify(&mut self, item: &Item) {

    }
}

pub fn update_state(state: &mut State) {
    let item = state.get_item(1);
    item.value += 10;

    state.further_modify(item);
}

fn main() {
    let mut state = State::new();
    update_state(&mut state);

    for (id, item) in &state.items {
        println!("Item ID: {}, Name: {}, Value: {}", id, item.name, item.value);
    }
}
