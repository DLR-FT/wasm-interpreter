use core::marker::PhantomData;

use alloc::vec::Vec;

enum SlotContent<T> {
    Unoccupied { prev_unoccupied: Option<usize> },
    Occupied { item: T },
}

struct Slot<T> {
    generation: u64,
    content: SlotContent<T>,
}

/// A contigious data structure that never shrinks, but keeps track of lazily deleted elements so that when a new item is inserted, the lazily deleted place is reused.
/// Insertion, removal, and update are all of O(1) time complexity. Inserted elements can be (mutably) accessed or removed with the key returned when they were inserted.
/// Note: might panic in case of more than `u64::MAX` insert calls during runtime.
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    last_unoccupied: Option<usize>,
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            last_unoccupied: None,
        }
    }
}

pub struct SlotMapKey<T> {
    index: usize,
    generation: u64,
    phantom: PhantomData<T>,
}

impl<T> SlotMap<T> {
    pub fn insert(&mut self, item: T) -> SlotMapKey<T> {
        match self.last_unoccupied {
            Some(last_unoccupied) => {
                let slot = &mut self.slots[last_unoccupied];
                let SlotContent::<T>::Unoccupied { prev_unoccupied } = slot.content else {
                    unreachable!("last unoccupied slot in slotmap must be unoccupied")
                };
                self.last_unoccupied = prev_unoccupied;

                // ASSUMPTION: slot.generation can never overflow.
                slot.generation = slot.generation.checked_add(1).unwrap();
                slot.content = SlotContent::Occupied { item };
                SlotMapKey::<T> {
                    index: last_unoccupied,
                    generation: slot.generation,
                    phantom: PhantomData,
                }
            }
            None => {
                let index = self.slots.len();
                let generation = 0;
                self.slots.push(Slot {
                    generation,
                    content: SlotContent::Occupied { item },
                });
                SlotMapKey::<T> {
                    index,
                    generation,
                    phantom: PhantomData,
                }
            }
        }
    }

    pub fn get(&self, key: &SlotMapKey<T>) -> Option<&T> {
        let slot = self.slots.get(key.index)?;
        if slot.generation != key.generation {
            return None;
        }
        match &slot.content {
            SlotContent::<T>::Unoccupied { .. } => None,
            SlotContent::<T>::Occupied { item } => Some(item),
        }
    }

    pub fn get_mut(&mut self, key: &SlotMapKey<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(key.index)?;
        if slot.generation != key.generation {
            return None;
        }
        match &mut slot.content {
            SlotContent::<T>::Unoccupied { .. } => None,
            SlotContent::<T>::Occupied { item } => Some(item),
        }
    }

    pub fn remove(&mut self, key: &SlotMapKey<T>) -> Option<T> {
        let slot = self.slots.get(key.index)?;
        if slot.generation != key.generation
            || matches!(slot.content, SlotContent::Unoccupied { .. })
        {
            return None;
        }
        let new_slot = Slot {
            generation: slot.generation,
            content: SlotContent::Unoccupied {
                prev_unoccupied: self.last_unoccupied,
            },
        };
        self.last_unoccupied = Some(key.index);
        self.slots.push(new_slot);
        let slot = self.slots.swap_remove(key.index);
        let SlotContent::Occupied { item } = slot.content else {
            unreachable!("slot was full")
        };
        Some(item)
    }
}

#[test]
fn test() {
    let mut slotmap = SlotMap::<i32>::default();
    let key = slotmap.insert(5);
    assert_eq!(slotmap.get(&key), Some(&5));
    slotmap.remove(&key);
    assert_eq!(slotmap.get(&key), None);
    let key2 = slotmap.insert(10);
    assert_eq!(slotmap.get(&key), None);
    assert_eq!(slotmap.get(&key2), Some(&10));
    let n = slotmap.get_mut(&key2).unwrap();
    *n = 42;
    assert_eq!(slotmap.get(&key2), Some(&42));
}
