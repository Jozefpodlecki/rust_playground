use std::collections::HashMap;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[derive(Debug)]
struct Entity {
    id: u32,
    name: String,
}

struct EntityHandle(NonNull<Entity>);

impl EntityHandle {
    fn new(entity: Entity) -> Self {
        let boxed = Box::new(entity);
        Self(NonNull::new(Box::into_raw(boxed)).unwrap())
    }
}

impl Deref for EntityHandle {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.0.as_ref() }
    }
}

impl DerefMut for EntityHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl Drop for EntityHandle {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.0.as_ptr()));
        }
    }
}

pub struct State {
    by_id: HashMap<u32, EntityHandle>,
    by_name: HashMap<String, *const Entity>,   
}

impl State {
    pub fn new() -> Self {
        State {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity) {
        let handle = EntityHandle::new(entity);
        let entity_ref = &*handle;
    
        let id = entity_ref.id;
        let name = entity_ref.name.clone();
    
        let raw_ptr = entity_ref as *const Entity;
    
        self.by_id.insert(id, handle);
        self.by_name.insert(name, raw_ptr);
    }

    pub fn get_by_id(&self, id: u32) -> Option<&Entity> {
        self.by_id.get(&id).map(|h| &**h)
    }
    
    pub fn get_by_name(&self, name: &str) -> Option<&Entity> {
        self.by_name.get(name).map(|&ptr| unsafe { &*ptr })
    }

    pub fn remove_by_id(&mut self, id: u32) -> Option<Entity> {
        if let Some(handle) = self.by_id.remove(&id) {
            let entity_ref = &handle;
            self.by_name.remove(&entity_ref.name);
            let raw = handle.0.as_ptr();
            mem::forget(handle);
            unsafe { Some(*Box::from_raw(raw)) }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut state = State::new();

        let entity = Entity {
            id: 1,
            name: "Alice".to_string(),
        };

        state.insert(entity);

        let by_id = state.get_by_id(1).unwrap();
        assert_eq!(by_id.id, 1);
        assert_eq!(by_id.name, "Alice");

        let by_name = state.get_by_name("Alice").unwrap();
        assert_eq!(by_name.id, 1);
        assert_eq!(by_name.name, "Alice");
    }

    #[test]
    fn test_remove_by_id() {
        let mut state = State::new();

        let entity = Entity {
            id: 42,
            name: "Zorg".to_string(),
        };

        state.insert(entity);
        assert!(state.get_by_id(42).is_some());
        assert!(state.get_by_name("Zorg").is_some());

        let removed = state.remove_by_id(42).unwrap();
        assert_eq!(removed.id, 42);
        assert_eq!(removed.name, "Zorg");

        assert!(state.get_by_id(42).is_none());
        assert!(state.get_by_name("Zorg").is_none());
    }

    #[test]
    fn test_multiple_entities() {
        let mut state = State::new();

        state.insert(Entity { id: 1, name: "Alpha".into() });
        state.insert(Entity { id: 2, name: "Beta".into() });
        state.insert(Entity { id: 3, name: "Gamma".into() });

        assert_eq!(state.get_by_name("Alpha").unwrap().id, 1);
        assert_eq!(state.get_by_name("Beta").unwrap().id, 2);
        assert_eq!(state.get_by_id(3).unwrap().name, "Gamma");

        let removed = state.remove_by_id(2).unwrap();
        assert_eq!(removed.name, "Beta");

        assert!(state.get_by_id(2).is_none());
        assert!(state.get_by_name("Beta").is_none());
    }
}
