use godot::prelude::*;
use godot::classes::{
    ArrayMesh, MeshInstance3D, MultiMeshInstance3D, RenderingServer, Shader, ShaderMaterial,
    Image, ImageTexture, MultiMesh,StandardMaterial3D,
    mesh::{PrimitiveType, ArrayType},
};
use ndarray::{Array2, s};
use rayon::iter::IntoParallelIterator;
use godot::classes::multi_mesh::TransformFormat;
use rust_data_table::SurvivalData;
use std::collections::HashSet;
use godot::classes::QuadMesh;
use godot::classes::SphereMesh;
use godot::classes::{Area3D, CollisionShape3D, BoxShape3D};

#[derive(GodotClass)]
#[class(base = Node3D, init)]
pub struct UmapGraph3D {
    #[base]
    base: Base<Node3D>,

    /// The dataset this projection belongs to
    #[export]
    pub dataset_name: GString,

    /// the label that can be shown above the 3D object
    #[export]
    pub label: GString,

    /// The projection type (e.g. "UMAP", "PCA")
    #[export]
    pub projection_type: GString,

    /// The number of clusters or groups visualized
    #[export]
    n_clusters: i32,

    /// Optional: if we need to look up this node from Rust logic later
    id: String,

    /// All the MultiMeshInstance3Ds used to render this projection
    meshes: Vec<Gd<MultiMeshInstance3D>>,

    /// The radius of the initial VR object.
    #[export]
    radius: f32,

    /// The re-centered center of the drc data.
    #[export]
    center: Vector3,

    /// The scale_factor the original data was scaled to fit into VR
    #[export]
    scale_factor:f32

}


#[godot_api]
impl UmapGraph3D {
    #[func]
    pub fn from_projection_data(
        &mut self,
        dataset_name: GString,
        projection_type: GString,
        path: GString,
        base_color: Color,
    ) {
        self.dataset_name = dataset_name.clone();
        self.projection_type = projection_type.clone();
        self.id = format!("{}::{}", dataset_name, projection_type);

        let path = path.to_string();

        let ds = match SurvivalData::from_tsv(
                &path,           // file
                b'\t',               // delimiter
                HashSet::new(),      // exclude set
                String::new(),       // dataset name or id if you use it
            ) {
            Ok(ds) => ds,
            Err(e) => {
                godot_error!("❌ Failed to read '{}': {}", path, e);
                return
            }
        };
        let n = ds.numeric_data.shape()[0];
        godot_print!("📊 building projection with {} points", n);
        let n_cols = ds.numeric_data.shape()[1];
        if n_cols < 4 {
            godot_print!("Dataset must have at least 3 numeric columns (x, y, z) + rownames;n_col = {}\n{path}\n{:?}",n_cols,ds.numeric_data.row(0) );
        }
        let view = ds.numeric_data.slice(s![.., 1..4]).mapv(|v| v as f32).to_owned();
        

        // ─── prepare MultiMesh
        let mut multimesh = MultiMesh::new_gd();
        multimesh.set_transform_format(TransformFormat::TRANSFORM_3D);
        multimesh.set_use_colors(true);        // 👈 THIS IS THE MAGIC LINE
        multimesh.set_instance_count(n as i32);

        let mut sphere = SphereMesh::new_gd();
        //quad.set_size(Vector2::new(1.0, 1.0));

        let radius = 0.005;
        sphere.set_radius(radius);
        sphere.set_height(radius * 2.0);
        sphere.set_radial_segments(8);
        sphere.set_rings(8);
        multimesh.set_mesh(&sphere);

        

        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for (i, row) in view.axis_iter(ndarray::Axis(0)).enumerate() {
            if row[0].is_nan() || row[1].is_nan() || row[2].is_nan() {
                continue;
            }
            let pos = Vector3::new(row[0] /10.0, row[1]/10.0, row[2]/10.0);

            // update bounds
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);

            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }

        // compute normalization
        // 2️⃣ compute center for X/Z and bottom-aligned Y
        self.center = Vector3::new((min.x + max.x) * 0.5, min.y, (min.z + max.z) * 0.5);

        // 3️⃣ compute uniform scale (fit in ~3 m space)
        let extent = (max - min).length();
        self.scale_factor = 3.0 / extent;

        for (i, row) in view.axis_iter(ndarray::Axis(0)).enumerate() {
            if row[0].is_nan() || row[1].is_nan() || row[2].is_nan() {
                multimesh.set_instance_color(i as i32, Color::from_rgba(0.0, 0.0, 0.0, 0.0)); // transparent
                multimesh.set_instance_transform(i as i32, Transform3D::IDENTITY.scaled(Vector3::ZERO)); // collapsed
                continue;
            }
            let pos = Vector3::new(row[0] /10.0, row[1]/10.0, row[2]/10.0);
            let pos_norm = (pos - self.center) * self.scale_factor * 3.0; // 3.0 = target size in meters

            let t = Transform3D::IDENTITY.translated(pos_norm);
            multimesh.set_instance_transform(i as i32, t);
            multimesh.set_instance_color(i as i32, base_color);
        }
        
        // ─── shader for round discs
        let mut mat: Gd<ShaderMaterial> = load("res://materials/umap_cells_std.tres");
        mat.set_shader_parameter("global_size", &1.0_f32.to_variant());

        // ─── instance node
        let mut inst = MultiMeshInstance3D::new_alloc();
        //inst.set_mulitmesh(&mesh);
        inst.set_multimesh(&multimesh);
        inst.set_material_override(&mat);

        self.base_mut().add_child(&inst);
        self.meshes.clear();
        self.meshes.push(inst);

        // create collision shape

        let mut area = Area3D::new_alloc();
        let name = GString::from("GrabArea");
        area.set_name(name.arg());

        let mut shape_node = CollisionShape3D::new_alloc();
        let mut shape = BoxShape3D::new_gd();
        let center = (min + max) * 0.5;
        let half_extents = (max - min) * 0.5;
        shape.set_size(half_extents);  // box extents (half-size)
        shape_node.set_shape(&shape);
        area.add_child(&shape_node);
        self.base_mut().add_child(&area);
        self.base_mut().add_to_group("UmapGraphs");

        godot_print!("✅ projection '{}'::'{}' ready ({} points)", dataset_name, projection_type, n);

    }


    pub fn world_selection_to_data_selection(
        &self,
        center_vr: Vector3,
        radius_vr: f32,
    ) -> (Vector3, f32) {
        godot_print!(
            "🧩 world_selection_to_data_selection called for {:?} with r={:.3}",
            center_vr,
            radius_vr,
        );

        // Convert world (VR) → local (graph) coordinates
        let transform = self.base().get_global_transform();
        let center_local = transform.affine_inverse().xform_inv(center_vr);

        // Convert graph local → data coordinate space
        let center_data = (center_local / self.scale_factor) + self.center;
        let radius_data = radius_vr / self.scale_factor;

        godot_print!(
            "📈 mapped to data-space: center={:?}, radius={:.4}",
            center_data,
            radius_data
        );

        (center_data, radius_data)
    }

    pub fn set_to_color(&mut self, ids: &Vec<i32>, color: Color) {
        if self.meshes.is_empty() {
            godot_warn!("⚠️ No MultiMesh found in UmapGraph3D '{}'", self.dataset_name);
            return;
        }

        // Assume first mesh holds the instances

        let inst = self.meshes[0].clone();
        let color_variant = color.to_variant();
        if let Some(mut mm) = inst.get_multimesh() {
            for id in ids.as_slice() {
                let idx = *id as i32;
                let c = mm.get_instance_color(idx);
                if c.a != 0.0 {
                    mm.set_instance_color(idx, color);
                }
            }
        }

        godot_print!(
            "🎨 Updated {} points to color ({:.2}, {:.2}, {:.2}) in '{}'",
            ids.len(),
            color.r,
            color.g,
            color.b,
            self.dataset_name
        );
    }


    /* // needs Godot update to get there!
    #[func]
    pub fn from_projection_data(
        &mut self,
        dataset_name: GString,
        projection_type: GString,
        view: &ndarray::Array2<f32>,
        color_opt: Option<Color>,        // <── new optional color argument
    ) {

        unsafe fn upload_cells_raw(cells: &[CellUVec4]) -> sys::RID {
            let rd = sys::godot_get_singleton(b"RenderingServer\0".as_ptr() as _);
            let rd = rd as *mut sys::RenderingDevice;  // ⚠️ unsafe raw pointer

            let size = (cells.len() * std::mem::size_of::<CellUVec4>()) as u64;
            let bytes = std::slice::from_raw_parts(
                cells.as_ptr() as *const u8,
                cells.len() * std::mem::size_of::<CellUVec4>(),
            );

            let mut rid: sys::RID = std::mem::zeroed();
            sys::RenderingDevice_storage_buffer_create(rd, &mut rid, size, 0);
            sys::RenderingDevice_storage_buffer_update(rd, &mut rid, 0, bytes.as_ptr(), size);
            rid
        }
        self.view = view.clone();
        self.dataset_name = dataset_name.clone();
        self.projection_type = projection_type.clone();
        self.id = format!("{}::{}", dataset_name, projection_type);

        let n = view.shape()[0];
        let mut cells = Vec::with_capacity(n);

        // --- build packed per-cell records ---
        let color = color_opt.unwrap_or(Color::from_rgb(0.9, 0.9, 0.9));
        let rgb = [
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
        ];

        for i in 0..n {
            let row = view.row(i);
            let x = row[0];
            let y = row[1];
            let z = row[2];
            let id = i as u32;
            cells.push(CellUVec4::new(x, y, z, rgb, id, 0.0_f32));
        }

        // --- upload to GPU as SSBO ---
        let buffer_rid = upload_cells(&RenderingServer::singleton(), &cells);

        // --- assign shader & uniforms ---
        let shader: Gd<Shader> = load("res://materials/pointcloud_uvec4_uniform.rdshader");
        let mut mat = ShaderMaterial::new_gd();
        mat.set_shader(&shader);
        mat.set_shader_parameter("pc/global_scale",  &Variant::from(1.0_f32));
        mat.set_shader_parameter("pc/global_size", &Variant::from(8.0_f32));

        RenderingServer::singleton()
            .material_set_storage_buffer(&mat, 1, 0, buffer_rid);

        // --- create a dummy mesh with N points ---
        let mesh = build_point_arraymesh(n);
        let mut mesh_instance = Gd::<MeshInstance3D>::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_material_override(&mat);

        self.base_mut().add_child(&mesh_instance);

        godot_print!(
            "✅ Rendered '{}'::'{}' with {} cells (uniform color {:?})",
            dataset_name,
            projection_type,
            n,
            color_opt
        );
    }*/


}

#[godot_api]
impl INode3D for UmapGraph3D {
    fn ready(&mut self) {
        godot_print!(
            "✅ UmapGraph3D ready (dataset={}, projection={}, meshes={})",
            self.dataset_name,
            self.projection_type,
            self.meshes.len(),
        );
        // Optional: show a label on startup
        //self.show_label(true);
    }
}

