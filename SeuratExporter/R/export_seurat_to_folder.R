#' Export a Seurat object to a Cell Ranger-style folder
#'
#' This function exports the counts matrix, feature and barcode tables,
#' cell metadata, and UMAP/PCA embeddings to a specified output directory.
#'
#' @param seurat_obj A Seurat object.
#' @param outdir Output directory (created if missing).
#' @param include_pca Logical; include PCA coordinates as `.drc` file. Default TRUE.
#'
#' @return Nothing. Files are written to disk.
#' @examples
#' \dontrun{
#' library(Seurat)
#' pbmc <- SeuratData::LoadData("pbmc3k")
#' export_seurat_to_folder(pbmc, "pbmc3k_export/")
#' }
#' @export
export_seurat_to_folder <- function(seurat_obj, outdir, include_pca = TRUE) {
  if (!requireNamespace("Matrix", quietly = TRUE))
    stop("Please install 'Matrix' package")
  if (!requireNamespace("R.utils", quietly = TRUE))
    stop("Please install 'R.utils' package")

  dir.create(outdir, showWarnings = FALSE, recursive = TRUE)

  ## --- 1. Extract count matrix
  counts <- Seurat::GetAssayData(seurat_obj, layer = "counts")
  counts <- as(counts, "dgCMatrix")

  ## --- 2. Write matrix.mtx.gz
  Matrix::writeMM(counts, file.path(outdir, "matrix.mtx"))
  R.utils::gzip(file.path(outdir, "matrix.mtx"), overwrite = TRUE)

  ## --- 3. Write features.tsv.gz
  features <- data.frame(
    gene_id = rownames(counts),
    gene_name = rownames(counts),
    feature_type = "Gene Expression"
  )
  write.table(features,
              file = gzfile(file.path(outdir, "features.tsv.gz")),
              sep = "\t", quote = FALSE, row.names = FALSE, col.names = FALSE)

  ## --- 4. Write barcodes.tsv.gz
  barcodes <- colnames(counts)
  writeLines(barcodes, gzfile(file.path(outdir, "barcodes.tsv.gz")))

  ## --- 5. Write meta.tsv
  meta <- seurat_obj@meta.data
  meta$barcode <- rownames(meta)
  write.table(meta, file = file.path(outdir, "meta.tsv"),
              sep = "\t", quote = FALSE, row.names = FALSE)

  ## --- 6. Write UMAP embedding
  for (name in names(seurat_obj@reductions)){
    umap <- Seurat::Embeddings(seurat_obj, name )
    tmp = umap[,1:2]
    if (ncol(umap) > 2 )
      tmp = umap[,1:3] # assuming we have velocity in here
    else {
      tmp = rbind(tmp, rep(0, nrow(tmp)))
    }

    write.table(tmp, file = file.path(outdir, paste0(name,".drc")),
                sep = "\t", quote = FALSE, col.names = FALSE)
    print( paste0("exported file '", paste0(name,".drc"),"'") )
  }

  message("✅ Export completed to ", outdir)
}
