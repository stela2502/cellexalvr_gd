use godot::prelude::*;

use godot::classes::{MeshInstance3D, SphereMesh, Node3D, StandardMaterial3D};

#[derive(GodotClass)]
#[class(base=Node3D, init)]
pub struct PrintForgeCore{
    /// Handle to the underlying Node3D Godot creates for this object.
    #[base]
    base: Base<Node3D>,
    /// a storage for the umap data. 
    umaps: HashMap<String, Gd<MultiMeshInstance3D>>,
}

#[godot_api]
impl PrintForgeCore{
    #[func]
    fn add_printable_sphere(&mut self, radius: f32, pos: Vector3) {
        godot_print!("🧱 adding sphere r={} at {:?}", radius, pos);

        // Create a visible Godot sphere
        let mut mesh = MeshInstance3D::new_alloc();
        let mut sphere_mesh = SphereMesh::new_gd();
        sphere_mesh.set_radius(radius);
        mesh.set_mesh(&sphere_mesh.upcast::<godot::classes::Mesh>());
        mesh.set_position(pos);

        // Attach to this node in the scene tree
        self.base_mut().add_child(&mesh);
    }


    #[func]
    fn make_hollow_sphere(&mut self, outer: f32, inner: f32, pos: Vector3, output: GString) {
        godot_print!(
            "🛠️ Generating hollow sphere outer={}mm inner={}mm → {}",
            outer,
            inner,
            output
        );

        // Create Godot built-in sphere mesh (for preview)
        let mut mesh_instance =  MeshInstance3D::new_alloc();
        let mut sphere = SphereMesh::new_gd();

        sphere.set_radius(outer);
        sphere.set_height(outer * 2.0);
        sphere.set_radial_segments(64);
        sphere.set_rings(32);

        mesh_instance.set_mesh(&sphere.upcast::<godot::classes::Mesh>());

        // Give it some color so you can see it
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgb(1.0, 0.4, 0.1)); // orange
        mesh_instance.set_surface_override_material(0, &mat);
        mesh_instance.set_position(pos);

        // Add to the current scene tree (attach to this node)
        self.base_mut().add_child(&mesh_instance);
    }
}
