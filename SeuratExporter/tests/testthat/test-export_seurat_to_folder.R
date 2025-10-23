test_that("export_seurat_to_folder creates expected files", {
  skip_if_not_installed("SeuratData")
  skip_if_not_installed("Seurat")

  library(Seurat)
  library(SeuratData)

  # Load example dataset
  pbmc <- SeuratData::LoadData("pbmc3k")

  tmpdir <- tempfile("pbmc_export_")
  dir.create(tmpdir)

  expect_no_error({
    export_seurat_to_folder(pbmc, tmpdir)
  })

  # Expected files
  expected_files <- c(
    "matrix.mtx.gz",
    "barcodes.tsv.gz",
    "features.tsv.gz",
    "meta.tsv",
    "umap.drc"
  )

  for (f in expected_files) {
    expect_true(file.exists(file.path(tmpdir, f)), info = paste("Missing:", f))
    expect_gt(file.size(file.path(tmpdir, f)), 10, info = paste("Empty:", f))
  }

  # PCA is optional, so only check if present
  pca_path <- file.path(tmpdir, "pca.drc")
  if (file.exists(pca_path)) {
    expect_gt(file.size(pca_path), 10)
  }
})
