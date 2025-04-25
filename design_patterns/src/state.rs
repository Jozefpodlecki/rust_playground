use std::{collections::HashMap, hash::Hash, collections::hash_map::Entry};

pub struct Item {
    name: String,
    value: u64,
}

pub struct Items(HashMap<u64, Item>);

impl Items {
    pub fn get(&mut self, id: u64) -> &mut Item {
        match self.0.entry(id) {
            Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            Entry::Vacant(vacant_entry) => vacant_entry.insert(Item { name: id.to_string(), value: 0 }),
        }
    }
}

pub struct State {
    items: Items
}

impl State {
    pub fn new() -> Self {
        let mut items = HashMap::new();
        items.insert(1, Item { name: "1".to_string(), value: 1 });
        let items = Items(items);

        Self { items }
    }

    pub fn further_modify(&mut self, value: u64) {

    }
}

pub fn update_state(state: &mut State) {
    let items = &mut state.items;
    let item = items.get(1);
    item.value += 10;

    let value = item.value;
    state.further_modify(value);
}

fn main() {
    let mut state = State::new();
    update_state(&mut state);

}
