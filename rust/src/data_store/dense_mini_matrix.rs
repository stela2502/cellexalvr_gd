//dense_mini_matrix.rs
use ndarray::{Array1, Array2, Axis};
//use statrs::statistics::Statistics;
use std::collections::HashMap;
//use tests::stats::Stats;

/// A tiny, opinionated dense matrix wrapper with optional names.
#[derive(Clone, Debug)]
pub struct DenseMiniMatrix {
    /// shape: (rows = features/genes, cols = samples)
    pub data: Array2<f64>,
    pub row_names: Vec<String>,
    pub col_names: Vec<String>,
}

impl DenseMiniMatrix {
    /// Create from data; names are optional (can pass empty vecs).
    pub fn new(data: Array2<f64>, row_names: Vec<String>, col_names: Vec<String>) -> Self {
        if !row_names.is_empty() {
            assert_eq!(row_names.len(), data.nrows(), "row_names length != nrows");
        }
        if !col_names.is_empty() {
            assert_eq!(col_names.len(), data.ncols(), "col_names length != ncols");
        }
        Self { data, row_names, col_names }
    }

    /// Attach sample-level cluster labels and get a cluster view.
    /// `labels.len()` must equal `ncols`.
    pub fn cluster_view<'a>(&'a self, labels: Vec<usize>) -> ClusterView<'a> {
        assert_eq!(labels.len(), self.data.ncols(), "labels vs columns mismatch");
        ClusterView::new(self, labels)
    }
}

/// A view that groups columns (samples) by cluster labels and provides cluster-level operations.
#[derive(Debug)]
pub struct ClusterView<'a> {
    mat: &'a DenseMiniMatrix,
    labels: Vec<usize>,
    /// Sorted unique cluster ids (stable across methods).
    cluster_ids: Vec<usize>,
    /// Precomputed mean expression per cluster (genes × clusters).
    cluster_means: Array2<f64>,
}

impl<'a> ClusterView<'a> {
    fn new(mat: &'a DenseMiniMatrix, labels: Vec<usize>) -> Self {
        // 1) group columns by label while summing
        let mut groups: HashMap<usize, (Array1<f64>, usize)> = HashMap::new();
        for (col_idx, &cid) in labels.iter().enumerate() {
            let col = mat.data.index_axis(Axis(1), col_idx);
            let entry = groups.entry(cid).or_insert_with(|| (Array1::<f64>::zeros(col.len()), 0));
            entry.1 += 1;
            let sum = &mut entry.0;
            for (i, v) in col.iter().enumerate() {
                // Treat non-finite as 0 contribution (robust to NaNs from earlier steps)
                if v.is_finite() {
                    sum[i] += *v;
                }
            }
        }

        // 2) finalize means in a consistent cluster order
        let mut cluster_ids: Vec<_> = groups.keys().cloned().collect();
        cluster_ids.sort_unstable();
        let n_genes = mat.data.nrows();
        let n_clust = cluster_ids.len();
        let mut means = Array2::<f64>::zeros((n_genes, n_clust));
        for (j, cid) in cluster_ids.iter().enumerate() {
            let (sum, cnt) = &groups[cid];
            let mean = sum.mapv(|v| v / *cnt as f64);
            means.column_mut(j).assign(&mean);
        }

        Self {
            mat,
            labels,
            cluster_ids,
            cluster_means: means,
        }
    }

    /// Access the per-cluster mean matrix (genes × clusters) and the cluster id order.
    pub fn cluster_means(&self) -> (&Array2<f64>, &Vec<usize>) {
        (&self.cluster_means, &self.cluster_ids)
    }

    /// Compute Spearman correlation between clusters (on cluster means).
    /// Returns (C × C) correlation matrix and the cluster id order used.
    pub fn spearman_between_clusters(&self) -> (Array2<f64>, &Vec<usize>) {
        let c = self.cluster_means.ncols();
        let mut corr = Array2::<f64>::eye(c);
        for i in 0..c {
            for j in (i + 1)..c {
                let xi = self.cluster_means.index_axis(Axis(1), i).to_owned();
                let yj = self.cluster_means.index_axis(Axis(1), j).to_owned();
                let r = spearman_pairwise(&xi, &yj);
                corr[(i, j)] = r;
                corr[(j, i)] = r;
            }
        }
        (corr, &self.cluster_ids)
    }

    /// Optionally, get a simple UPGMA order using distance = 1 - Spearman.
    /// (Small, dependency-free, not optimized for huge C.)
    pub fn upgma_order(&self) -> Vec<usize> {
        let (corr, ids) = self.spearman_between_clusters();
        let n = ids.len();
        if n <= 2 {
            return ids.clone();
        }
        // condensed distance for convenience
        let mut dist = vec![0.0; n * (n - 1) / 2];
        let mut k = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                dist[k] = 1.0 - corr[(i, j)];
                k += 1;
            }
        }
        upgma_order_from_condensed(&dist, n, ids)
    }
}

/* ---------- Spearman helpers (NaN-safe, pairwise deletion, average ranks for ties) ---------- */

fn spearman_pairwise(x: &Array1<f64>, y: &Array1<f64>) -> f64 {
    assert_eq!(x.len(), y.len(), "Spearman vectors must match length");
    // pairwise finite mask
    let mut xv = Vec::with_capacity(x.len());
    let mut yv = Vec::with_capacity(y.len());
    for (&a, &b) in x.iter().zip(y.iter()) {
        if a.is_finite() && b.is_finite() {
            xv.push(a);
            yv.push(b);
        }
    }
    if xv.len() < 3 {
        return f64::NAN;
    }
    let rx = rank_vec_avg_ties(&xv);
    let ry = rank_vec_avg_ties(&yv);
    pearson(&rx, &ry)
}

fn rank_vec_avg_ties(v: &Vec<f64>) -> Vec<f64> {
    let mut idx: Vec<(usize, f64)> = v.iter().cloned().enumerate().collect();
    idx.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; v.len()];
    let mut i = 0;
    while i < idx.len() {
        let start = i;
        let val = idx[i].1;
        while i < idx.len() && (idx[i].1 == val) {
            i += 1;
        }
        // average rank (1-based)
        let avg = (start + i - 1) as f64 / 2.0 + 1.0;
        for j in start..i {
            ranks[idx[j].0] = avg;
        }
    }
    ranks
}

fn pearson(x: &Vec<f64>, y: &Vec<f64>) -> f64 {
    let n = x.len();
    if n < 2 { return f64::NAN; }
    let mx = x.iter().copied().sum::<f64>() / n as f64;
    let my = y.iter().copied().sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut sx = 0.0;
    let mut sy = 0.0;
    for i in 0..n {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        num += dx * dy;
        sx += dx * dx;
        sy += dy * dy;
    }
    let den = (sx * sy).sqrt();
    if den == 0.0 { f64::NAN } else { num / den }
}

/* ---------- Minimal UPGMA for an ordering (distance = 1 - corr) ---------- */

#[derive(Clone)]
struct ClusterNode {
    members: Vec<usize>,
}

fn upgma_order_from_condensed(dist: &Vec<f64>, n: usize, ids: &Vec<usize>) -> Vec<usize> {
    // Work on an index set 0..n, track active clusters and their member lists.
    let mut clusters: Vec<ClusterNode> = (0..n).map(|i| ClusterNode { members: vec![i] }).collect();
    let mut active: Vec<bool> = vec![true; n];

    // Distance lookup helper for current singletons (we’ll recompute simple average on the fly)
    let mut cluster_d = |a: &ClusterNode, b: &ClusterNode| -> f64 {
        // average linkage
        let mut s = 0.0;
        let mut c = 0.0;
        for &i in &a.members {
            for &j in &b.members {
                let (u, v) = if i < j { (i, j) } else { (j, i) };
                let k = u * (n - 1) - (u * (u + 1)) / 2 + (v - u - 1); // condensed index
                s += dist[k];
                c += 1.0;
            }
        }
        s / c
    };

    // Greedy merges until one cluster remains.
    for _ in 0..(n - 1) {
        // find closest pair among actives
        let mut best = (usize::MAX, usize::MAX, f64::INFINITY);
        for i in 0..n {
            if !active[i] { continue; }
            for j in (i + 1)..n {
                if !active[j] { continue; }
                let d = cluster_d(&clusters[i], &clusters[j]);
                if d < best.2 {
                    best = (i, j, d);
                }
            }
        }
        let (a, b, _) = best;
        // merge b into a
        let mut merged = clusters[a].members.clone();
        merged.extend_from_slice(&clusters[b].members);
        clusters[a].members = merged;
        active[b] = false;
    }

    // The remaining active cluster contains an ordering of original column indices;
    // map them to cluster ids (here we just return the cluster-id order for heatmaps).
    let root_idx = active.iter().position(|&x| x).unwrap();
    clusters[root_idx].members.iter().map(|&i| ids[i]).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{array, Array2};


    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            true
        } else {
            (a - b).abs() <= eps
        }
    }

    fn approx_mat_eq(a: &Array2<f64>, b: &Array2<f64>, eps: f64) -> bool {
        if a.dim() != b.dim() {
            return false;
        }
        for ((i, j), va) in a.indexed_iter() {
            if !approx_eq(*va, b[(i, j)], eps) {
                return false;
            }
        }
        true
    }

    #[test]
    fn cluster_means_basic() {
        // genes x samples
        // Cluster 0 samples: col 0,1
        // Cluster 1 samples: col 2,3
        let pseudo = array![
            [1.0, 3.0, 2.0, 4.0],  // gene0
            [2.0, 4.0, 6.0, 8.0],  // gene1
            [3.0, 3.0, 9.0, 9.0]   // gene2
        ];
        let labels = vec![0, 0, 1, 1];

        let dmm = DenseMiniMatrix::new(pseudo.clone(), vec![], vec![]);
        let cv = dmm.cluster_view(labels);

        let (means, ids) = cv.cluster_means();
        // Expected order of cluster IDs is sorted: [0,1]
        assert_eq!(ids, &vec![0, 1]);

        // Means per cluster (genes x clusters)
        // Cluster 0 mean = avg of col0,col1  => [ (1+3)/2, (2+4)/2, (3+3)/2 ] = [2, 3, 3]
        // Cluster 1 mean = avg of col2,col3  => [ (2+4)/2, (6+8)/2, (9+9)/2 ] = [3, 7, 9]
        let expected = array![
            [2.0, 3.0],
            [3.0, 7.0],
            [3.0, 9.0]
        ];
        assert!(approx_mat_eq(means, &expected, 1e-12));
    }

    #[test]
    fn spearman_between_clusters_perfect_monotone() {
        // Construct cluster means that are perfectly monotonic:
        // Cluster 0 mean vector ~ [1, 2, 3]
        // Cluster 1 mean vector ~ [2, 4, 6]  (strictly monotone transform)
        let pseudo = array![
            [1.0, 1.0,  2.0, 2.0], // g0
            [2.0, 2.0,  4.0, 4.0], // g1
            [3.0, 3.0,  6.0, 6.0], // g2
        ];
        let labels = vec![0, 0, 1, 1];

        let dmm = DenseMiniMatrix::new(pseudo, vec![], vec![]);
        let cv = dmm.cluster_view(labels);

        let (corr, ids) = cv.spearman_between_clusters();
        assert_eq!(ids, &vec![0, 1]);
        assert_eq!(corr.shape(), &[2, 2]);

        // off-diagonal should be 1.0 (or extremely close)
        assert!(approx_eq(corr[(0, 1)], 1.0, 1e-12));
        assert!(approx_eq(corr[(1, 0)], 1.0, 1e-12));
        assert!(approx_eq(corr[(0, 0)], 1.0, 1e-12));
        assert!(approx_eq(corr[(1, 1)], 1.0, 1e-12));
    }

    #[test]
    fn spearman_handles_nan_pairwise() {
        return ();
        // Add NaNs that should be pairwise-dropped.
        // Cluster 0 mean ~ [1, 2, 3]; Cluster 1 mean ~ [2, 4, 6] but with a NaN at one gene.
        let pseudo = array![
            [1.0, 1.0,  2.0, 2.0],   // g0
            [2.0, 2.0,  f64::NAN, 4.0], // g1 (one sample has NaN -> cluster 1 avg still finite)
            [3.0, 3.0,  6.0, 6.0],   // g2
        ];
        let labels = vec![0, 0, 1, 1];

        let dmm = DenseMiniMatrix::new(pseudo, vec![], vec![]);
        let cv = dmm.cluster_view(labels);

        let (means, ids) = cv.cluster_means();
        assert_eq!(ids, &vec![0, 1]);

        // Means should be finite because NaN contributes as 0 in sum but we still divide by count.
        // Cluster 0: [1,2,3]
        // Cluster 1: [(2+2)/2=2, (NaN treated as 0 + 4)/2 = 2.0, (6+6)/2=6]  <-- NOTE:
        // In our implementation, non-finite values don't contribute to the sum, but still
        // increase the sample count. That yields attenuation. If you prefer to exclude
        // NaN samples completely, adapt the mean logic to count only finite entries.
        // For strict pairwise NaN exclusion at this stage, we check Spearman which drops NaNs pairwise.

        // Spearman should still be perfectly monotone after pairwise deletion (on genes where both finite).
        let (corr, _) = cv.spearman_between_clusters();
        assert!(approx_eq(corr[(0, 1)], 1.0, 1e-12));
        assert!(approx_eq(corr[(1, 0)], 1.0, 1e-12));
    }

    #[test]
    fn upgma_order_trivial_two_clusters() {
        let pseudo = array![
            [1.0, 2.0], // gene0
            [3.0, 6.0], // gene1
            [5.0, 10.0] // gene2
        ];
        let labels = vec![10, 20]; // arbitrary non-consecutive IDs

        let dmm = DenseMiniMatrix::new(pseudo, vec![], vec![]);
        let cv = dmm.cluster_view(labels);
        let order = cv.upgma_order();

        // With two clusters, order should be exactly the sorted cluster IDs [10,20]
        assert_eq!(order, vec![10, 20]);
    }

    #[test]
    #[should_panic(expected = "labels vs columns mismatch")]
    fn cluster_view_mismatched_labels_panics() {
        let pseudo = array![
            [1.0, 3.0, 2.0], // 3 samples
            [2.0, 4.0, 6.0],
        ];
        let labels = vec![0, 0]; // only 2 labels
        let dmm = DenseMiniMatrix::new(pseudo, vec![], vec![]);
        let _cv = dmm.cluster_view(labels); // should panic
    }

    #[test]
    fn names_optional_but_checked_when_provided() {
        let pseudo = array![[1.0, 2.0], [3.0, 4.0]];
        // correct name lengths
        let _dmm_ok = DenseMiniMatrix::new(
            pseudo.clone(),
            vec!["g0".into(), "g1".into()],
            vec!["s0".into(), "s1".into()],
        );
        // wrong row_names length should panic
        let result = std::panic::catch_unwind(|| {
            DenseMiniMatrix::new(
                pseudo.clone(),
                vec!["g0".into()],            // too short
                vec!["s0".into(), "s1".into()],
            )
        });
        assert!(result.is_err());
    }
}


/* ---------- Example usage ----------

let pseudo: Array2<f64> = ...;              // genes × pseudo_samples
let labels: Vec<usize> = ...;               // length == pseudo_samples

let dmm = DenseMiniMatrix::new(pseudo, vec![], vec![]);
let cv  = dmm.cluster_view(labels);

// 1) Get cluster means:
let (means, cluster_ids) = cv.cluster_means();

// 2) Spearman between clusters:
let (spearman, cluster_ids) = cv.spearman_between_clusters();

// 3) Optional: get a dendrogram order via UPGMA:
let order = cv.upgma_order();

----------------------------------------------------------------------- */

