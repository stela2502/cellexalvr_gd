use godot::prelude::*;
use std::collections::HashMap;

use crate::data_store::DataStore;
use crate::umap_graph_3d::UmapGraph3D;

#[derive(GodotClass)]
#[class(base = Node3D, init)]
pub struct PrintForgeCore {
    #[base]
    base: Base<Node3D>,

    datasets: HashMap<String, DataStore>,
    projections: HashMap<(String, String), Gd<UmapGraph3D>>,
}

#[godot_api]
impl PrintForgeCore {

    #[func]
    pub fn load_dataset_and_projections(&mut self, name: GString, path: GString) {
        // 1️⃣ Call your original core loader
        self.load_dataset(name.clone(), path.clone());

        // 2️⃣ Then search for projections and visualize them
        self.load_local_projections(name, path);
    }


    fn load_dataset(&mut self, name: GString, path: GString) {
        godot_print!("📂 Loading dataset '{}' from {}", name, path);
        match DataStore::from_cellranger(&path.to_string()) {
            Ok(ds) => {
                self.datasets.insert(name.to_string(), ds);
                godot_print!("✅ Dataset '{}' loaded", name);
            }
            Err(e) => godot_error!("❌ Failed to load dataset '{}': {}", name, e),
        }

    }

    fn load_local_projections(&mut self, name: GString, path: GString) {
        use std::path::Path;
        use std::fs;
        let path = path.to_string();
        let dataset_path = Path::new(&path);
        let dataset_str = dataset_path
            .file_name()                    // last path component
            .and_then(|s| s.to_str())       // convert OsStr → &str
            .unwrap_or("<unknown>");        // fallback if not valid UTF-8

        let mut projections = Vec::new();

        if let Ok(entries) = fs::read_dir(dataset_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("drc") || ext.eq_ignore_ascii_case("tsv") {
                        projections.push(entry.path());
                    }
                }
            }
        }

        for proj_path in projections {
            let proj_type = proj_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

            godot_print!("📈 Found projection {}", proj_type);

            // Spawn a new UmapGraph3D node
            let mut graph = UmapGraph3D::new_alloc();
            graph.bind_mut().setup(
                dataset_str.into(),
                (&proj_type).into(),
                10,
                (&proj_path.to_string_lossy().to_string()).into(),
            );
            // Attach to scene so it renders
            self.base_mut().add_child(&graph);

            // Register it in projections map
            self.projections.insert((dataset_str.to_string(), proj_type.to_string()), graph);

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