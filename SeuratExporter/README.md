SeuratExporter

A lightweight R package to export Seurat objects into a **Cell Ranger–compatible folder structure**.

## Installation

```r
# Install from local folder
devtools::install("SeuratExporter")

# or load for development
devtools::load_all("SeuratExporter")
```

## Example

```r
library(SeuratExporter)
library(Seurat)

# Download (if not already installed)
SeuratData::InstallData("pbmc3k")

pbmc <- SeuratData::LoadData("pbmc3k")


# Standard workflow to generate reductions
pbmc <- NormalizeData(pbmc)
pbmc <- FindVariableFeatures(pbmc)
pbmc <- ScaleData(pbmc)
pbmc <- RunPCA(pbmc)
pbmc <- RunUMAP(pbmc, dims = 1:10, n.components=3)
pbmc <- FindNeighbors(pbmc, dims = 1:10)
pbmc <- FindClusters(pbmc, resolution = 0.5)


export_seurat_to_folder(pbmc, "pbmc3k/")
```

This will produce:

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
| `meta.tsv` | Seurat object metadata |
| `umap.drc` | UMAP embedding coordinates |
| `pca.drc` | PCA embedding coordinates |

---

Licensed under MIT.
