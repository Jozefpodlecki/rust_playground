use std::{collections::HashMap, hash::Hash, collections::hash_map::Entry};

#[derive(Debug, Default)]
pub struct Item {
    name: String,
    value: u64,
}

pub struct Items(HashMap<u64, Item>);

impl Items {
    pub fn get_mut(&mut self, id: u64) -> &mut Item {
        match self.0.entry(id) {
            Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            Entry::Vacant(vacant_entry) => vacant_entry.insert(Item { name: id.to_string(), value: 0 }),
        }
    }

    pub fn get_mut_item(&mut self, id: u64) -> &mut Item {
        let item_ptr = self.0.get_mut(&id).unwrap() as *mut Item;
        unsafe { &mut *item_ptr }
    }
    
}

pub struct State {
    items: Items
}

impl State {
    pub fn new() -> Self {
        let mut items = HashMap::new();
        items.insert(1, Item { name: "1".to_string(), value: 1 });
        items.insert(2, Item { name: "2".to_string(), value: 2 });
        let items = Items(items);

        Self { items }
    }

    pub fn modify(&mut self, item: &Item, target: &Item, owner: &Item) {
        println!("{item:?}");
        println!("{target:?}");
        println!("{owner:?}");
    }
}

pub fn update_state(state: &mut State) {
    let items = &mut state.items;

    let source = unsafe { &mut *(items.get_mut(1) as *mut Item) };
    let target = unsafe { &mut *(items.get_mut(2) as *mut Item) };

    let owner = &mut Item { name: "3".into(), value: 3 };

    state.modify(source, target, owner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut state = State::new();
        update_state(&mut state);
    }
}