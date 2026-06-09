#![allow(dead_code)]

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId {
    index: u32,
    generation: u32,
}

enum Slot<T> {
    Occupied {
        value: T,
        generation: u32,
    },
    Free {
        next_free: Option<u32>,
        generation: u32,
    },
}

pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<u32>,
    len: usize,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> NodeId {
        if let Some(index) = self.free_head {
            let slot = &mut self.slots[index as usize];
            if let Slot::Free {
                next_free,
                generation,
            } = *slot
            {
                self.free_head = next_free;
                *slot = Slot::Occupied { value, generation };
                self.len += 1;
                return NodeId { index, generation };
            }
        }

        let index = self.slots.len() as u32;
        let generation = 1;
        self.slots.push(Slot::Occupied { value, generation });
        self.len += 1;
        NodeId { index, generation }
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        match slot {
            Slot::Occupied { value, generation } if *generation == id.generation => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        match slot {
            Slot::Occupied { value, generation } if *generation == id.generation => Some(value),
            _ => None,
        }
    }

    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        match slot {
            Slot::Occupied { generation, .. } if *generation == id.generation => {
                let next_generation = generation.wrapping_add(1);
                let old_slot = std::mem::replace(
                    slot,
                    Slot::Free {
                        next_free: self.free_head,
                        generation: next_generation,
                    },
                );

                self.free_head = Some(id.index);
                self.len -= 1;

                if let Slot::Occupied { value, .. } = old_slot {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.get(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut arena = Arena::new();
        let id1 = arena.insert("a");
        let id2 = arena.insert("b");

        assert_eq!(arena.len(), 2);
        assert_eq!(arena.get(id1), Some(&"a"));
        assert_eq!(arena.get(id2), Some(&"b"));
    }

    #[test]
    fn test_get_mut() {
        let mut arena = Arena::new();
        let id = arena.insert(10);
        if let Some(val) = arena.get_mut(id) {
            *val = 20;
        }
        assert_eq!(arena.get(id), Some(&20));
    }

    #[test]
    fn test_remove_stale_detection() {
        let mut arena = Arena::new();
        let id = arena.insert("item");
        assert_eq!(arena.remove(id), Some("item"));
        assert_eq!(arena.len(), 0);
        assert_eq!(arena.get(id), None);
        assert!(!arena.contains(id));
    }

    #[test]
    fn test_generation_reuse() {
        let mut arena = Arena::new();
        let id1_v1 = arena.insert("first");
        arena.remove(id1_v1);

        let id1_v2 = arena.insert("second");
        // Index should be reused
        assert_eq!(id1_v1.index, id1_v2.index);
        // Generation should be different
        assert_ne!(id1_v1.generation, id1_v2.generation);

        // Old ID should be stale
        assert_eq!(arena.get(id1_v1), None);
        // New ID should work
        assert_eq!(arena.get(id1_v2), Some(&"second"));
    }

    #[test]
    fn test_is_empty() {
        let mut arena: Arena<i32> = Arena::new();
        assert!(arena.is_empty());
        let id = arena.insert(1);
        assert!(!arena.is_empty());
        arena.remove(id);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_multiple_removals_and_insertions() {
        let mut arena = Arena::new();
        let id1 = arena.insert(1);
        let id2 = arena.insert(2);
        let id3 = arena.insert(3);

        arena.remove(id2);
        let id4 = arena.insert(4);
        assert_eq!(id4.index, id2.index);
        assert_ne!(id4.generation, id2.generation);

        arena.remove(id1);
        arena.remove(id3);
        assert_eq!(arena.len(), 1);

        let id5 = arena.insert(5);
        let id6 = arena.insert(6);
        assert!(id5.index == id3.index || id5.index == id1.index);
        assert!(id6.index == id3.index || id6.index == id1.index);
    }
}
