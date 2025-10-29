use godot::prelude::*;
use std::sync::{Arc, RwLock};

use crate::drc::umap_graph_3d::UmapGraph3D;
use crate::vr::graph_interaction_registry::GraphInteractionRegistry;
use crate::vr::grabbable::Grabbable;

/// A VR hand that can touch and grab `Grabbable` objects.
///
/// This node should be a child of an `XRController3D` in Godot.
/// It listens for `Area3D` overlaps and uses the shared registry
/// to coordinate grab interactions.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct VrHand {
    #[base]
    base: Base<Node3D>,

    #[export]
    pub hand_id: i32, // 0 = left, 1 = right

    // Shared registry (thread-safe)
    registry: Arc<RwLock<GraphInteractionRegistry<UmapGraph3D>>>,

    // Currently overlapping object
    touched_object: Option<Gd<UmapGraph3D>>,

    // Whether grip button is pressed
    is_grabbing: bool,
}

#[godot_api]
impl VrHand {
    #[func]
    pub fn init(&mut self, registry: Arc<RwLock<GraphInteractionRegistry<UmapGraph3D>>>) {
        self.registry = registry;
    }

    /// Called by Godot when the hand enters a grabbable's Area3D
    #[func]
    pub fn on_area_entered(&mut self, area: Gd<Area3D>) {
        if let Some(parent) = area.get_parent() {
            if let Ok(graph) = parent.try_cast::<UmapGraph3D>() {
                godot_print!("✋ Hand {} touching {:?}", self.hand_id, graph.get_name());
                self.touched_object = Some(graph);
            }
        }
    }

    /// Called when hand leaves a grabbable area
    #[func]
    pub fn on_area_exited(&mut self, _area: Gd<Area3D>) {
        self.touched_object = None;
    }

    /// Called every frame
    #[func]
    pub fn _process(&mut self, _delta: f64) {
        let hand_transform = self.base().get_global_transform();

        // grip input (depending on your controller mapping)
        let grip_pressed = Input::singleton().is_action_pressed(
            if self.hand_id == 0 { "left_grip" } else { "right_grip" },
        );

        match (grip_pressed, self.is_grabbing, self.touched_object.clone()) {
            // Start grabbing
            (true, false, Some(obj)) => {
                if let Ok(mut graph) = obj.clone().try_cast::<UmapGraph3D>() {
                    graph.bind_mut().on_grab_start(self.hand_id as usize, hand_transform);
                }
                self.registry
                    .write()
                    .unwrap()
                    .hand_grabs(self.hand_id as usize, obj);
                self.is_grabbing = true;
            }

            // Release
            (false, true, Some(_)) => {
                self.registry.write().unwrap().hand_releases(self.hand_id as usize);
                if let Some(obj) = &self.touched_object {
                    if let Ok(mut graph) = obj.clone().try_cast::<UmapGraph3D>() {
                        graph.bind_mut().on_grab_release(self.hand_id as usize);
                    }
                }
                self.is_grabbing = false;
            }

            _ => {}
        }
    }
}