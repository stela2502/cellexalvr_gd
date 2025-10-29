use godot::prelude::*;
use std::collections::HashMap;

use crate::vr::grabbable::Grabbable;

/// Tracks which hands are currently grabbing which objects.
///
/// This lives inside your main VR interaction context (or shared Arc<Mutex<...>>),
/// so both hands can update it.

pub struct GraphInteractionRegistry<T: Grabbable + GodotClass<Base = Node3D>> {
    hand_to_object: HashMap<usize, Gd<T>>,
}

impl<T: Grabbable + GodotClass<Base = Node3D>> GraphInteractionRegistry<T> {
    pub fn new() -> Self {
        Self {
            hand_to_object: HashMap::new(),
        }
    }

    /// A hand grabs a specific object.
    pub fn hand_grabs(&mut self, hand_id: usize, object: Gd<T>) {
        self.hand_to_object.insert(hand_id, object);
    }

    /// A hand releases whatever it holds.
    pub fn hand_releases(&mut self, hand_id: usize) {
        self.hand_to_object.remove(&hand_id);
    }

    /// Update all held objects: call their `on_grab_move()` using
    /// current transforms for each hand.
    pub fn update(&mut self, hand_transforms: &[(usize, Transform3D)]) {
        for (hand_id, transform) in hand_transforms {
            if let Some(obj) = self.hand_to_object.get_mut(hand_id) {
                let mut bound = obj.bind_mut();
                bound.on_grab_move(&[*transform]);
            }
        }
    }

    /// Query which object a hand currently holds.
    pub fn object_for_hand(&self, hand_id: usize) -> Option<Gd<T>> {
        self.hand_to_object.get(&hand_id).cloned()
    }
}