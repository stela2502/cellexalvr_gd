use rust_data_table::SurvivalData;
use sprs::CsMat;
use std::collections::HashSet;

#[derive(Debug)]
pub struct DataStore {
    pub counts: CsMat<f32>,      // expression matrix (genes × cells)
    pub gene_names: Vec<String>, // from features.tsv.gz
    pub cell_names: Vec<String>, // from barcodes.tsv.gz
    pub meta: SurvivalData,      // all annotations and cluster info
}

impl DataStore {
    /// this initializes the data view in VR
    /// at the moment it expects simple text files:
    /// features.tsv.gz, barcodes.tsv.gz and matrix.mtx.gz for the expression data
    /// meta.tsv - a table containing the cell meta info (will be parsed by SurvivalData!)
    /// meta.factors.json a file ulimtately created by the VR process definig how the data in the meat sould be used 
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
        let tri: sprs::TriMat<f32> = sprs::io::read_matrix_market_from_bufread(&mut reader)?;
        let counts = tri.to_csr();

        // --- Initialize empty metadata table ---
        let mut meta = SurvivalData::from_file("meta.tsv", b'\t', HashSet::<String>::new(), "meta.factors.json" );
        //meta.add_column("cell_id", cell_names.iter().map(|s| s.as_str()), true)?;

        Ok(Self {
            counts,
            gene_names,
            cell_names,
            meta:meta?,
        })
    }

}
