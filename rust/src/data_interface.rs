use survival_data::SurvivalData;
use sprs::CsMat;

#[derive(Debug)]
pub struct DataStore {
    pub counts: CsMat<f32>,      // expression matrix (genes × cells)
    pub gene_names: Vec<String>, // from features.tsv.gz
    pub cell_names: Vec<String>, // from barcodes.tsv.gz
    pub meta: SurvivalData,      // all annotations and cluster info
}

impl DataStore {
    pub fn from_cellranger<P: AsRef<std::path::Path>>(dir: P) -> anyhow::Result<Self> {
        let dir = dir.as_ref();

        // --- Gene names ---
        let f = std::fs::File::open(dir.join("features.tsv.gz"))?;
        let decoder = flate2::read::GzDecoder::new(f);
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(decoder);
        let gene_names = rdr
            .records()
            .map(|r| r.unwrap()[1].to_string())
            .collect::<Vec<_>>();

        // --- Cell barcodes ---
        let f = std::fs::File::open(dir.join("barcodes.tsv.gz"))?;
        let decoder = flate2::read::GzDecoder::new(f);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(decoder);
        let cell_names = rdr
            .records()
            .map(|r| r.unwrap()[0].to_string())
            .collect::<Vec<_>>();

        // --- Matrix ---
        let f = std::fs::File::open(dir.join("matrix.mtx.gz"))?;
        let decoder = flate2::read::GzDecoder::new(f);
        let mut reader = std::io::BufReader::new(decoder);
        let tri: sprs::TriMat<f32> = sprs::io::read_matrix_market(&mut reader)?;
        let counts = tri.to_csr();

        // --- Initialize empty metadata table ---
        let mut meta = SurvivalData::new();
        meta.add_column("cell_id", cell_names.iter().map(|s| s.as_str()), true)?;

        Ok(Self {
            counts,
            gene_names,
            cell_names,
            meta,
        })
    }
    
    /// Add a new numeric or categorical vector to the metadata
    pub fn add_meta_column(&mut self, name: &str, values: Vec<String>) -> anyhow::Result<()> {
        assert_eq!(values.len(), self.cell_names.len());
        self.meta.add_column(name, values.iter().map(|s| s.as_str()), false)?;
        Ok(())
    }

    pub fn add_numeric_column(&mut self, name: &str, values: Vec<f32>) -> anyhow::Result<()> {
        assert_eq!(values.len(), self.cell_names.len());
        let vals = values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
        self.meta.add_column(name, vals.iter().map(|s| s.as_str()), false)?;
        Ok(())
    }
}
