
/*
    loads a flat PackedFloat32Array of positions [x0,y0,z0, x1,y1,z1, ...]
    builds a MultiMeshInstance3D with a tiny sphere mesh
    turns on custom data and uses it for selection (the shader reads it)
    exposes an update_selection(box_pos, box_size) method
*/

use godot::prelude::*;
use godot::classes::{MultiMesh, MultiMeshInstance3D, SphereMesh, Mesh, Node3D, StandardMaterial3D};

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct UmapViewer {
    #[base]
    base: Base<Node3D>,

    positions: Vec<Vector3>,
    multimesh: Option<Gd<MultiMesh>>,
}

#[godot_api]
impl UmapViewer {
    #[func]
    fn load_umap(&mut self, flat_xyz: PackedFloat32Array) {
        let n = flat_xyz.len() / 3;
        self.positions = (0..n)
            .map(|i| Vector3::new(flat_xyz[i*3], flat_xyz[i*3+1], flat_xyz[i*3+2]))
            .collect();

        // Base sphere mesh (keep it very low poly)
        let mut sphere = SphereMesh::new_gd();
        sphere.set_radius(0.01);
        sphere.set_radial_segments(8);
        sphere.set_rings(8);

        // MultiMesh
        let mut mm = MultiMesh::new_gd();
        mm.set_mesh(sphere.upcast::<Mesh>());
        mm.set_use_custom_data(true);     // <-- enable per-instance custom data!
        mm.set_instance_count(self.positions.len() as i32);

        for (i, p) in self.positions.iter().enumerate() {
            mm.set_instance_transform(i as i32, Transform3D::from_translation(*p));
            // default: not selected → INSTANCE_CUSTOM.r = 0.0
            mm.set_instance_custom_data(i as i32, Color::from_rgba(0.0, 0.0, 0.0, 1.0));
        }

        let mut mm_instance = MultiMeshInstance3D::new_alloc();
        mm_instance.set_multimesh(&mm);

        // Optional: set a plain material here; we’ll override in Godot with the shader
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_transparency(StandardMaterial3D::TRANSPARENCY_DISABLED);
        mm_instance.set_material_override(mat.upcast());

        self.base_mut().add_child(&mm_instance);
        self.multimesh = Some(mm);

        godot_print!("✅ UMAP loaded: {} points", n);
    }

    /// Update selection by axis-aligned box (center + size in world space, *relative to viewer* space if parented)
    #[func]
    fn update_selection(&mut self, box_center: Vector3, box_size: Vector3) {
        if let Some(ref mm) = self.multimesh {
            let half = box_size * 0.5;
            let min = box_center - half;
            let max = box_center + half;

            for (i, p) in self.positions.iter().enumerate() {
                let sel = (p.x >= min.x && p.x <= max.x) &&
                          (p.y >= min.y && p.y <= max.y) &&
                          (p.z >= min.z && p.z <= max.z);

                // Write selection state into INSTANCE_CUSTOM.r (0.0 or 1.0)
                let val = if sel { 1.0 } else { 0.0 };
                mm.set_instance_custom_data(i as i32, Color::from_rgba(val, 0.0, 0.0, 1.0));
            }
        }
    }
}

