anndata_exporter

Export AnnData objects into a **Cell Ranger–like folder** structure.

## Installation

```bash
pip install .
```

## Example

```python
import scanpy as sc
from anndata_exporter import export_anndata_to_folder

adata = sc.datasets.pbmc3k()
export_anndata_to_folder(adata, "pbmc3k_export/")
```

Output:

```
pbmc3k_export/
├── matrix.mtx.gz
├── barcodes.tsv.gz
├── features.tsv.gz
├── meta.tsv
├── umap.drc
└── pca.drc
```

## Output Files

| File | Description |
| --- | --- |
| `matrix.mtx.gz` | Sparse count matrix (genes × cells) |
| `barcodes.tsv.gz` | Cell barcodes |
| `features.tsv.gz` | Gene names and feature types |
| `meta.tsv` | AnnData object metadata |
| `umap.drc` | UMAP embedding coordinates |
| `pca.drc` | PCA embedding coordinates |

---

Licensed under MIT.
