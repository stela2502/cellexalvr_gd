use godot::prelude::*;
use std::collections::HashMap;

use crate::data_store::DataStore;
use crate::umap_graph_3d::UmapGraph3D;
use godot::classes::Engine;
use std::path::Path;
use std::fs;


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