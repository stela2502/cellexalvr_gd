//grabbable.rs

//! Defines the `Grabbable` trait — implemented by any object that can be
//! grabbed, moved, or scaled by VR hands.
//!
//! Godot (via `Area3D` + `CollisionShape3D`) handles collision detection.
//! This trait defines *what happens* once a grab begins.
//!
//! Typical implementations include:
//! - [`UmapGraph3D`](crate::drc::umap_graph_3d::UmapGraph3D)
//! - Interactive tools, sliders, 3D UI elements, etc.
//!
//! `VrHand` or a central interaction system calls these methods when
//! a hand enters a grabbable's area and grip input is pressed.

use godot::prelude::*;

/// Trait for any Rust-based Godot object that can be manipulated by VR hands.
///
/// Implement this on your 3D data or tool nodes to define how they react to
/// being grabbed, moved, or released.
///
/// The VR hands will call these methods automatically when:
/// - `on_grab_start`: Grip is pressed while hand overlaps this object
/// - `on_grab_move`: One or both hands are moving while holding
/// - `on_grab_release`: Grip released
pub trait Grabbable: Send + Sync {
    /// Called when a VR hand first grabs this object.
    ///
    /// - `hand_id`: unique hand index (e.g., 0 = left, 1 = right)
    /// - `hand_transform`: world-space transform of the grabbing hand at grab start
    fn on_grab_start(&mut self, hand_id: usize, hand_transform: Transform3D);

    /// Called every frame while the object is being held.
    ///
    /// - `hand_transforms`: a slice containing one or two transforms.
    ///   - If 1 → one-hand grab → translate/rotate the object.
    ///   - If 2 → two-hand grab → scale/rotate between both.
    fn on_grab_move(&mut self, hand_transforms: &[Transform3D]);

    /// Called when a VR hand releases the object.
    ///
    /// - `hand_id`: index of the hand releasing the grab.
    fn on_grab_release(&mut self, hand_id: usize);
}
