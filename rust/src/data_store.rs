use rust_data_table::SurvivalData;
use std::collections::HashSet;
use sprs::io::read_matrix_market_from_bufread;
use sprs::{CsMat, TriMat};
use ndarray::{Array2, s,Axis };
use std::io::BufReader;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::path::Path;

use std::fs::{self,File};

#[derive(Debug)]
pub struct DataStore {
    pub counts: CsMat<f32>,      // expression matrix (genes × cells)
    pub gene_names: Vec<String>, // from features.tsv.gz
    pub cell_names: Vec<String>, // from barcodes.tsv.gz
    pub cell_meta: SurvivalData, // all annotations and cluster info
    pub gene_meta: SurvivalData, // in case we want to store some info there later
    pub drcs: HashMap<String, Array2<f32>>,
    active_group: Option<String>,
    group_id:usize,
}

impl DataStore {
    /// this initializes the data view in VR
    /// at the moment it expects simple text files:
    /// features.tsv.gz, barcodes.tsv.gz and matrix.mtx.gz for the expression data
    /// meta.tsv - a table containing the cell meta info (will be parsed by SurvivalData!)
    /// meta.factors.json a file ulimtately created by the VR process definig how the data in the meat sould be used 
    pub fn from_cellranger<P: AsRef<std::path::Path>>(dir: P) -> Result<Self, String> {
        let dir = dir.as_ref();

        // --- Gene names ---
        let features_path = dir.join("features.tsv.gz");
        let f = File::open(&features_path)
            .map_err(|e| format!("❌ Failed to open {:?}: {}", features_path, e))?;
        let decoder = GzDecoder::new(f);
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(decoder);
        let gene_names: Vec<String> = rdr
            .records()
            .filter_map(|r| r.ok())
            .map(|r| r[1].to_string())
            .collect();
        if gene_names.is_empty() {
            return Err(format!("❌ No gene names found in {:?}", features_path));
        }

        // --- Cell barcodes ---
        let barcodes_path = dir.join("barcodes.tsv.gz");
        let f = File::open(&barcodes_path)
            .map_err(|e| format!("❌ Failed to open {:?}: {}", barcodes_path, e))?;
        let decoder = GzDecoder::new(f);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(decoder);
        let cell_names: Vec<String> = rdr
            .records()
            .filter_map(|r| r.ok())
            .map(|r| r[0].to_string())
            .collect();
        if cell_names.is_empty() {
            return Err(format!("❌ No cell barcodes found in {:?}", barcodes_path));
        }

        // --- Matrix (.mtx.gz) ---
        let matrix_path = dir.join("matrix.mtx.gz");
        let tri: TriMat<f32> = {
            let f = File::open(&matrix_path)
                .map_err(|e| format!("❌ Failed to open {:?}: {}", matrix_path, e))?;
            let decoder = GzDecoder::new(f);
            let mut reader = BufReader::new(decoder);

            match read_matrix_market_from_bufread::<f32, usize, _>(&mut reader) {
                Ok(tri) => tri,
                Err(_) => {
                    // Retry as integer and cast to f32
                    let f = File::open(&matrix_path)
                        .map_err(|e| format!("❌ Failed to reopen {:?}: {}", matrix_path, e))?;
                    let decoder = GzDecoder::new(f);
                    let mut reader = BufReader::new(decoder);
                    let int_tri: TriMat<i32> = read_matrix_market_from_bufread::<i32, usize, _>(&mut reader)
                        .map_err(|e| format!("❌ Failed to parse {:?} as integer MatrixMarket: {}", matrix_path, e))?;

                    let mut tri_f32 = TriMat::<f32>::with_capacity(
                        (int_tri.rows(), int_tri.cols()),
                        int_tri.nnz(),
                    );
                    for (v, (r, c)) in int_tri.triplet_iter() {
                        tri_f32.add_triplet(r, c, *v as f32);
                    }
                    tri_f32
                }
            }
        };

        let counts: CsMat<f32> = tri.to_csr();
        if counts.nnz() == 0 {
            return Err(format!("❌ Matrix {:?} appears empty", matrix_path));
        }

        // --- Metadata ---
        let meta_path = dir.join("meta.tsv");
        let meta_json_path = dir.join("meta.factors.json");
        let cell_meta = SurvivalData::from_file(
            &meta_path,
            b'\t',
            HashSet::<String>::new(),
            &meta_json_path,
        )
        .map_err(|e| format!("❌ Failed to load metadata: {}", e))?;

        
        println!("📈 searching path {} for projections linke '*.drc'",dir.to_string_lossy() );
        let dataset_path = Path::new(&dir);
        let dataset_str = dataset_path
            .file_name()                    // last path component
            .and_then(|s| s.to_str())       // convert OsStr → &str
            .unwrap_or("<unknown>");        // fallback if not valid UTF-8

        let mut projections = Vec::new();

        if let Ok(entries) = fs::read_dir(dataset_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("drc") {
                        projections.push(entry.path());
                    }
                }
            }
        }
        println!("📈 Found projections {:?}", projections);


        let mut ret = Self{
            counts,
            gene_names,
            gene_meta: SurvivalData::default(),
            cell_names,
            cell_meta,
            drcs: HashMap::new(),
            active_group:None,
            group_id:0,
        };
        for proj_path in projections {
            let proj_type = proj_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
            if let Err(e) = ret.load_projection_from_tsv( &proj_type, &proj_path.to_string_lossy() ){
                println!("⚠️ Failed to load projection '{}': {}", proj_type, e);
            }
        }


        Ok(ret)
    }


    /// Load one DR coordinate file (UMAP, PCA, etc.) from TSV
    pub fn load_projection_from_tsv(&mut self, name: &str, path: &str) -> Result<(), String> {
        let ds = SurvivalData::from_tsv(path, b'\t', HashSet::new(), String::new())
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let barcode_factor = ds
            .factors
            .get(&ds.headers[0])
            .ok_or_else(|| "No barcode column found".to_string())?;

        if let Some(our_barcodes) = self.cell_meta.factors.get("barcode"){
            if let Err(e) = our_barcodes.compare_levels(barcode_factor) {
                return Err(format!("Barcode mismatch in projection '{}': {}", name, e));
            }
        } else {
            return Err("Current dataset has no 'barcode' factor in cellmeta".into());
        }

        let n_cols = ds.numeric_data.shape()[1];
        if n_cols < 4 {
            return Err("Dataset must have at least 3 numeric columns (x, y, z) + rownames".into());
        }

        let view = ds.numeric_data.slice(s![.., 1..4]).mapv(|v| v as f32).to_owned();

        self.drcs.insert(name.to_string(), view);
        Ok(())
    }

    pub fn get_projection(&self, name: &str) -> Option<&Array2<f32>> {
        self.drcs.get(name)
    }

    /// Select cells in a projection by 3D position + radius (VR-space),
    /// updating `cell_meta` and creating `active_group` if needed.
    pub fn select_cells(
        &mut self,
        projection_name: &str,
        group_id: usize,
        position: &[f32], // 3D position from VR
        radius: f32,      // VR radius (same scale as positions)
    ) -> anyhow::Result<()> {
        // ─── ensure projection exists
        let Some(view) = self.drcs.get(projection_name) else {
            anyhow::bail!("Projection '{}' not found", projection_name);
        };

        // ─── initialize active_group if needed
        if self.active_group.is_none() {
            let name = format!("group_{:03}", self.group_id);
            self.cell_meta.add_dataset(&name, true, None );
            let order = format!("group_{:03}_order", self.group_id);
            self.cell_meta.add_dataset(&order, false, None );
            self.active_group = Some(name);
            self.cell_meta.reset_order();
        }

        // the column name and its companion order column
        let group_name = self.active_group.clone().unwrap();
        let order_col = format!("{}_order", group_name);

        // ─── compute squared radius once
        let r2 = radius * radius;
        let center = [position[0], position[1], position[2]];

        // ─── scan through all cells and update metadata
        let mut changed: Vec<(usize, f32)> = Vec::new();
        for (i, row) in view.axis_iter(Axis(0)).enumerate() {
            let dx = row[0] - center[0];
            let dy = row[1] - center[1];
            let dz = row[2] - center[2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 <= r2 {
                // try to update; if changed, mark with order id
                if self.cell_meta.update_value(&group_name, i, group_id as f64 ) {
                    
                    changed.push((i, d2));
                }
            }
        }

        // sort by distance
        changed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, _) in changed {
            self.cell_meta.update_order(&order_col, i);
        }
        
        Ok(())
    }

}
