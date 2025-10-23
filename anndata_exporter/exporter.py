"""
Export AnnData object to Cell Ranger–style folder
"""
import os
import gzip
import numpy as np
import pandas as pd
from scipy.io import mmwrite
from anndata import AnnData

def export_anndata_to_folder(adata: AnnData, outdir: str, include_pca: bool = True):
    """
    Export AnnData object to a Cell Ranger–like folder with:
    matrix.mtx.gz, barcodes.tsv.gz, features.tsv.gz, meta.tsv, umap.drc, and pca.drc
    """
    os.makedirs(outdir, exist_ok=True)

    # 1. Counts matrix
    if adata.raw is not None:
        X = adata.raw.X
        var = adata.raw.var
    else:
        X = adata.X
        var = adata.var

    # Ensure sparse
    from scipy.sparse import issparse, csr_matrix
    if not issparse(X):
        X = csr_matrix(X)

    # matrix.mtx.gz
    mmwrite(os.path.join(outdir, "matrix.mtx"), X)
    with open(os.path.join(outdir, "matrix.mtx"), "rb") as f_in:
        with gzip.open(os.path.join(outdir, "matrix.mtx.gz"), "wb") as f_out:
            f_out.writelines(f_in)
    os.remove(os.path.join(outdir, "matrix.mtx"))

    # features.tsv.gz
    features = pd.DataFrame({
        "gene_id": var.index,
        "gene_name": var.index,
        "feature_type": "Gene Expression"
    })
    with gzip.open(os.path.join(outdir, "features.tsv.gz"), "wt") as f:
        features.to_csv(f, sep="\t", header=False, index=False)

    # barcodes.tsv.gz
    with gzip.open(os.path.join(outdir, "barcodes.tsv.gz"), "wt") as f:
        for bc in adata.obs_names:
            f.write(f"{bc}\n")

    # meta.tsv
    meta = adata.obs.copy()
    meta["barcode"] = adata.obs_names
    meta.to_csv(os.path.join(outdir, "meta.tsv"), sep="\t", index=False)

    # umap.drc
    if "X_umap" in adata.obsm:
        pd.DataFrame(adata.obsm["X_umap"], index=adata.obs_names).to_csv(
            os.path.join(outdir, "umap.drc"), sep="\t"
        )

    # pca.drc
    if include_pca and "X_pca" in adata.obsm:
        pd.DataFrame(adata.obsm["X_pca"], index=adata.obs_names).to_csv(
            os.path.join(outdir, "pca.drc"), sep="\t"
        )

    print(f"✅ Export completed to {outdir}")

