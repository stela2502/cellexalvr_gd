use godot::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::data_store::DataStore;
use crate::umap_graph_3d::UmapGraph3D;
use godot::classes::Engine;
use std::path::Path;
use std::fs;
use crate::utils::color_to_id;
use ordered_float::OrderedFloat;


#[derive(GodotClass)]
#[class(base = Node3D, init)]
pub struct PrintForgeCore {
    #[base]
    base: Base<Node3D>,

    datasets: HashMap<String, DataStore>,
    projections: Vec<Gd<UmapGraph3D>>,
}

#[godot_api]
impl PrintForgeCore {

    #[func]
    pub fn load_dataset_and_projections(&mut self, name: GString, path: GString) {
        let real_path = path.to_string();
        let name = name.to_string();

        // 1️⃣ Call your original core loader
        self.load_dataset(&name, &real_path);

        // 2️⃣ Then search for projections and visualize them

        let dataset_path =  Path::new(&real_path);
        godot_print!("Initializing 3D graphs");
        match fs::read_dir(dataset_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let Some(ext) = p.extension().and_then(|e| e.to_str()) else { continue };
                    if !ext.eq_ignore_ascii_case("drc") { continue }

                    // projection type from file stem, fallback to full name
                    let proj_name = p.file_stem()
                        .and_then(|s| Some(format!("{}",s.display())) )
                        .unwrap_or_else(|| p.to_string_lossy().to_string() )
                        .to_string();

                    let path_str: String = p.to_string_lossy().into_owned();
                    godot_print!("I have identiofied a file: '{}'", path_str);

                    let mut graph = UmapGraph3D::new_alloc();
                    graph.bind_mut().from_projection_data(
                        (&name).into(),              // dataset_name
                        (&proj_name).into(),         // projection_type
                        (&path_str).into(),                  // path to the .drc/.tsv file
                        Color::from_rgb(0.9, 0.9, 0.9),   // base color
                    );

                    self.base_mut().add_child(&graph);
                    //self.projections.push(graph);
                }
            }
            Err(e) => {
                godot_error!("❌ Could not read directory '{}': {}", dataset_path.display(), e);
            }
        }
        godot_print!("Finished");

    }


    fn load_dataset(&mut self, name: &str, path: &str) -> Option<Vec<String>> {
        godot_print!("📂 Rust: Loading dataset '{}' from {}", name, path);
        match DataStore::from_cellranger(path) {
            Ok(ds) => {
                godot_print!("✅ Dataset '{}' loaded", name);
                self.datasets.insert(name.to_string(), ds);
                self.datasets.get(name)
                    .and_then(|stored_ds| {
                        stored_ds.cell_meta
                            .factors
                            .get("barcode")
                            .map(|f| f.get_levels().to_vec())
                    })
            }
            Err(e) => {
                godot_error!("❌ Failed to load dataset '{}' from {}: {}", name, path, e);
                None
            },
        }

    }

    #[func]
    pub fn handle_selection(&mut self, center_data: Vector3, radius_data: f32, color: Color) {
        godot_print!(
            "🎯 Core handling selection center={:?}, r={:.3}",
            center_data,
            radius_data
        );

        let group_id = color_to_id(&color);


        for (dataset_name, ds) in self.datasets.iter_mut() {
            // Collect unique cell IDs across all projections for this dataset
            let mut all_selected: HashSet<i32> = HashSet::new();

            // 1️⃣ Iterate over all projections belonging to this dataset
            for   graph_gd in self.projections.iter() {
                let  graph = graph_gd.bind();

                if graph.dataset_name.to_string() != *dataset_name {
                    continue;
                }

                // Convert VR → data space for this graph
                let (data_center, data_radius) =
                    graph.world_selection_to_data_selection(center_data, radius_data);

                // Select once per dataset (if not yet computed)
                let pos = [center_data.x, center_data.y, center_data.z];
                let selected = ds.select_in_sphere(
                    &(graph.projection_type.to_string()),
                    &group_id,
                    &pos,
                    data_radius,

                );
                while let Ok(ref v) = selected{
                    all_selected.extend(v);
                }
            }

            // 2️⃣ Apply coloring to ALL UmapGraph3D belonging to this dataset
            if !all_selected.is_empty() {
                let selected_vec: Vec<i32> = all_selected.iter().copied().collect();
                for mut graph_gd in self.projections.iter_mut() {
                    let mut graph = graph_gd.bind_mut();
                    if graph.dataset_name.to_string() == *dataset_name {
                        graph.set_to_color(&selected_vec, color);
                    }
                }
            }
        }
    }

    
    fn matches_ignore_ascii_case(a: &str, b: &str) -> bool {
        a.eq_ignore_ascii_case(b)
    }

}

#[godot_api]
impl INode3D for PrintForgeCore {
    fn ready(&mut self) {
        godot_print!("🧠 PrintForgeCore ready — awaiting dataset load...");
    }
}