import os
import pytest
import scanpy as sc
from anndata_exporter.exporter import export_anndata_to_folder

def test_export(tmp_path):
    # small example from scanpy
    adata = sc.datasets.pbmc3k()  # requires internet
    outdir = tmp_path / "exported"
    export_anndata_to_folder(adata, outdir)

    expected = [
        "matrix.mtx.gz",
        "barcodes.tsv.gz",
        "features.tsv.gz",
        "meta.tsv",
        "umap.drc"
    ]

    for f in expected:
        path = outdir / f
        assert path.exists(), f"Missing {f}"
        assert path.stat().st_size > 100, f"{f} seems empty"

    # pca optional
    pca_path = outdir / "pca.drc"
    if pca_path.exists():
        assert pca_path.stat().st_size > 100

