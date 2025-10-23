//umap_graph.rs
use godot::prelude::*;
use std::collections::HashMap;
use rust_kmeans::DataSet;
use godot::classes::{ Label3D, MultiMeshInstance3D, MultiMesh, base_material_3d::BillboardMode};

#[derive(GodotClass)]
#[class(base = Node3D, init)]
pub struct UmapGraph3D {
    #[base]
    base: Base<Node3D>,

    /// The dataset this projection belongs to
    #[export]
    dataset_name: GString,

    /// the label that can be shown above the 3D object
    #[export]
    label: GString,

    /// The projection type (e.g. "UMAP", "PCA")
    #[export]
    projection_type: GString,

    /// The number of clusters or groups visualized
    #[export]
    n_clusters: i32,

    /// Optional: if we need to look up this node from Rust logic later
    id: String,

    /// All the MultiMeshInstance3Ds used to render this projection
    meshes: Vec<Gd<MultiMeshInstance3D>>,
}

#[godot_api]
impl UmapGraph3D {
    /// Initialize this UMAP 3D node.
    #[func]
    pub fn setup(&mut self, dataset_name: GString, projection_type: GString, n_clusters: i32, path: GString,) {
        self.dataset_name = dataset_name;
        self.projection_type = projection_type;
        self.n_clusters = n_clusters;

        self.id = format!(
            "{}::{}",
            self.dataset_name.to_string(),
            self.projection_type.to_string()
        );

        godot_print!("📂 Loading dataset via rust_kmeans from {}", path);
        
        // --- Load data ---
        let ds = match DataSet::from_tsv(&path.to_string()) {
            Ok(d) => d,
            Err(e) => {
                godot_error!("❌ Failed to load dataset: {}", e);
                return;
            }
        };

        // --- Store it --- not necessary!
        // self.data = Some(ds.clone());

        // --- Prepare positions ---
        let view = ds.numeric_view(6);
        let n_cols = view.shape()[1];
        if n_cols < 3 {
            godot_error!("❌ Dataset must have at least 3 numeric columns (x,y,z)");
            return;
        }

        let n_rows = view.shape()[0];
        let mut multimesh = MultiMesh::new_gd();
        multimesh.set_instance_count(n_rows as i32);
        multimesh.set_use_custom_data(true);

        let mut colors = vec![Color::from_rgb(0.7, 0.7, 0.7); n_rows];
        let mut velocities: Option<Vec<[f32; 3]>> = None;

        /*if n_cols >= 6 {
            velocities = Some(
                (0..n_rows)
                    .map(|i| {
                        [
                            view[[i, 3]],
                            view[[i, 4]],
                            view[[i, 5]],
                        ]
                    })
                    .collect(),
            );
        }*/

        // --- Optional clustering --- for later when we have a lot lot of cells
        // let clusters = ds.kmeans3d(n_clusters as usize, 100).unwrap_or_else(|_| vec![0; n_rows]);
        // let n_clusters = clusters.iter().max().unwrap_or(&0) + 1;

        // let cluster_colors = Self::generate_cluster_colors(n_clusters as usize);

        for i in 0..n_rows {
        	if view[[i, 0]].is_nan() || view[[i, 1]].is_nan() || view[[i, 2]].is_nan() {
		        continue; // 🚫 skip NA entries, keep ID alignment
		    }
            let pos = Vector3::new(view[[i, 0]], view[[i, 1]], view[[i, 2]]);
            let transform = Transform3D::IDENTITY.translated(pos);
            multimesh.set_instance_transform(i as i32, transform);

            let c = Color::from_rgb(0.7, 0.7, 0.7);
            multimesh.set_instance_custom_data(i as i32,c  );
        }

        // --- Add to scene ---
        let mut mesh_instance = MultiMeshInstance3D::new_alloc();
        mesh_instance.set_multimesh(&multimesh);
        self.base_mut().add_child(&mesh_instance);
        self.meshes.push(mesh_instance);

        //self.velocities = velocities;

        godot_print!(
            "✅ Loaded {} cells, {} clusters ({} cols)",
            n_rows, n_clusters, n_cols
        );
    }


    /// Add a new cluster (creates one MultiMeshInstance3D for that cluster)
    pub fn add_cluster(
        &mut self,
        positions: &[(f32, f32, f32)],
        colors: &[(f32, f32, f32)],
    ) -> Gd<MultiMeshInstance3D> {
        let mut multimesh = MultiMesh::new_gd();
        multimesh.set_use_custom_data(true);
        multimesh.set_instance_count(positions.len() as i32);

        for (i, (x, y, z)) in positions.iter().enumerate() {
            let transform = Transform3D::IDENTITY.translated(Vector3::new(*x, *y, *z));
            multimesh.set_instance_transform(i as i32, transform);

            let (r, g, b) = colors.get(i).copied().unwrap_or((1.0, 1.0, 1.0));
            let color = Color::from_rgb(r, g, b);
            multimesh.set_instance_custom_data(i as i32, color);
        }

        let mut mesh_instance = MultiMeshInstance3D::new_alloc();
        mesh_instance.set_multimesh(&multimesh);

        // add to scene
        self.base_mut().add_child(&mesh_instance);
        self.meshes.push(mesh_instance.clone());

        mesh_instance
    }

    /// Total number of points visualized
    #[func]
    pub fn count_cells(&self) -> i32 {
        self.meshes
            .iter()
            .map(|m| m
                .get_multimesh().map_or(0, |mm| mm.get_instance_count())
            )
            .sum::<i32>()
    }

    /* /// later
    /// Create or remove the hovering label
    #[func]
    pub fn show_label(&mut self, show: bool) {
        if show {
            if self.label.is_none() {
                let mut lbl = Gd::<Label3D>::new_alloc();
                let text = format!(
                    "{} — {}",
                    self.dataset_name.to_string(),
                    self.projection_type.to_string()
                );
                lbl.set_text(GString::from(&text));
                lbl.set_billboard_mode(godot::classes::label_3d::BillboardMode::ENABLED);

                // optional: adjust position slightly above the graph
                lbl.set_position(Vector3::new(0.0, 2.0, 0.0));

                // styling
                lbl.set_modulate(Color::from_rgb(0.8, 0.9, 1.0));
                lbl.set_font_size(24.0);

                self.base_mut().add_child(&lbl);
                self.label = Some(lbl);

                godot_print!("🪧 Added label for {} — {}", self.dataset_name, self.projection_type);
            }
        } else {
            if let Some(lbl) = self.label {
                self.base_mut().remove_child(&lbl);
                self.label = None;
                godot_print!("🧹 Removed label for {} — {}", self.dataset_name, self.projection_type);
            }
        }
    }
    */
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

