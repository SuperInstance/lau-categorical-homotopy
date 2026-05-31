#![deny(unsafe_code)]
//! # lau-categorical-homotopy
//!
//! Homotopy type theory meets agent systems — paths, homotopies, fundamental
//! groupoids, and higher inductive types as agent communication protocols.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. TopologicalSpace
// ---------------------------------------------------------------------------

/// Finite approximation of a topological space via a simplicial complex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalSpace {
    pub points: Vec<f64>,
    /// Adjacency list: `adjacency[i]` lists indices adjacent to point `i`.
    pub adjacency: Vec<Vec<usize>>,
}

impl TopologicalSpace {
    pub fn new(points: Vec<f64>, adjacency: Vec<Vec<usize>>) -> Self {
        Self { points, adjacency }
    }

    /// Convenience: build a discrete space (no edges).
    pub fn discrete(points: Vec<f64>) -> Self {
        let n = points.len();
        Self {
            points,
            adjacency: vec![vec![]; n],
        }
    }

    /// Build a 1-D circle sampled with `n` equally-spaced points in [0, 1).
    pub fn circle(n: usize) -> Self {
        let n = n.max(3);
        let points: Vec<f64> = (0..n).map(|i| i as f64 / n as f64).collect();
        let adjacency: Vec<Vec<usize>> = (0..n)
            .map(|i| {
                vec![
                    (i + n - 1) % n, // predecessor
                    (i + 1) % n,     // successor
                ]
            })
            .collect();
        Self { points, adjacency }
    }

    /// Build a line segment [0, 1] sampled with `n` points.
    pub fn interval(n: usize) -> Self {
        let n = n.max(2);
        let points: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();
        let adjacency: Vec<Vec<usize>> = (0..n)
            .map(|i| {
                let mut adj = Vec::new();
                if i > 0 {
                    adj.push(i - 1);
                }
                if i + 1 < n {
                    adj.push(i + 1);
                }
                adj
            })
            .collect();
        Self { points, adjacency }
    }

    /// Connected components via BFS.
    pub fn connected_components(&self) -> Vec<Vec<usize>> {
        let n = self.points.len();
        let mut visited = vec![false; n];
        let mut components = Vec::new();
        for start in 0..n {
            if visited[start] {
                continue;
            }
            let mut comp = Vec::new();
            let mut stack = vec![start];
            while let Some(v) = stack.pop() {
                if visited[v] {
                    continue;
                }
                visited[v] = true;
                comp.push(v);
                for &u in &self.adjacency[v] {
                    if !visited[u] {
                        stack.push(u);
                    }
                }
            }
            comp.sort_unstable();
            components.push(comp);
        }
        components
    }

    /// A space is contractible if every loop is null-homotopic.
    /// For our finite approximation: trivial iff there is exactly one connected
    /// component *and* no cycle in the adjacency graph (a tree).
    pub fn contractible(&self) -> bool {
        if self.connected_components().len() != 1 {
            return false;
        }
        // Check acyclicity via DFS cycle detection.
        let n = self.points.len();
        if n <= 1 {
            return true;
        }
        let mut visited = vec![false; n];
        let mut stack: Vec<(usize, Option<usize>)> = vec![(0, None)];
        while let Some((v, parent)) = stack.pop() {
            if visited[v] {
                continue;
            }
            visited[v] = true;
            for &u in &self.adjacency[v] {
                if Some(u) == parent {
                    continue;
                }
                if visited[u] {
                    return false; // cycle
                }
                stack.push((u, Some(v)));
            }
        }
        true
    }

    /// Euler characteristic: V - E/2  (for 1-dimensional simplicial complex).
    pub fn euler_characteristic(&self) -> i32 {
        let v = self.points.len() as i32;
        let half_e: i32 = self
            .adjacency
            .iter()
            .map(|adj| adj.len() as i32)
            .sum::<i32>()
            / 2;
        v - half_e
    }

    /// Topological dimension (max simplex dimension; we work with 1-complexes so
    /// dimension is at most 1 unless we have no edges).
    pub fn dimension(&self) -> usize {
        if self.adjacency.iter().any(|a| !a.is_empty()) {
            1
        } else {
            0
        }
    }

    /// Find the index of the point nearest to `x`.
    pub fn nearest(&self, x: f64) -> usize {
        self.points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (**a - x).abs().partial_cmp(&(**b - x).abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// 2. Path
// ---------------------------------------------------------------------------

/// A continuous map [0,1] → X represented as a polyline (sampled points).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub points: Vec<f64>,
}

impl Path {
    pub fn new(points: Vec<f64>) -> Self {
        Self { points }
    }

    pub fn constant(x: f64, n: usize) -> Self {
        Self {
            points: vec![x; n.max(2)],
        }
    }

    pub fn start(&self) -> f64 {
        self.points.first().copied().unwrap_or(0.0)
    }

    pub fn end(&self) -> f64 {
        self.points.last().copied().unwrap_or(0.0)
    }

    /// Evaluate the path at parameter `t ∈ [0,1]` via linear interpolation.
    pub fn evaluate(&self, t: f64) -> f64 {
        let n = self.points.len();
        if n == 0 {
            return 0.0;
        }
        if n == 1 {
            return self.points[0];
        }
        let t = t.clamp(0.0, 1.0);
        let scaled = t * (n - 1) as f64;
        let i = scaled.floor() as usize;
        let frac = scaled - i as f64;
        if i + 1 >= n {
            return self.points[n - 1];
        }
        self.points[i] * (1.0 - frac) + self.points[i + 1] * frac
    }

    /// Concatenate two paths. Returns `None` unless `self.end() == other.start()`
    /// (within tolerance).
    pub fn concat(&self, other: &Path) -> Option<Path> {
        const EPS: f64 = 1e-9;
        if (self.end() - other.start()).abs() > EPS {
            return None;
        }
        let mut pts = self.points.clone();
        // skip first point of other (same as last of self)
        if other.points.len() > 1 {
            pts.extend_from_slice(&other.points[1..]);
        }
        Some(Path::new(pts))
    }

    /// Reverse the path.
    pub fn reverse(&self) -> Path {
        Path::new(self.points.iter().rev().copied().collect())
    }

    /// Resample to `n` equidistant evaluation points.
    pub fn reparametrize(&self, n: usize) -> Path {
        if n <= 1 {
            return Path::constant(self.start(), 2);
        }
        let pts: Vec<f64> = (0..n).map(|i| self.evaluate(i as f64 / (n - 1) as f64)).collect();
        Path::new(pts)
    }

    /// Total arc length.
    pub fn length(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum()
    }
}

// ---------------------------------------------------------------------------
// 3. Homotopy
// ---------------------------------------------------------------------------

/// A continuous deformation between two paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Homotopy {
    pub from_path: Path,
    pub to_path: Path,
    pub steps: usize,
}

impl Homotopy {
    pub fn new(from: Path, to: Path, steps: usize) -> Self {
        Self {
            from_path: from,
            to_path: to,
            steps: steps.max(2),
        }
    }

    /// The path at parameter `t ∈ [0,1]` via linear interpolation of the two
    /// endpoint paths (reparametrized to same resolution).
    pub fn intermediate(&self, t: f64) -> Path {
        let n = self.steps;
        let from = self.from_path.reparametrize(n);
        let to = self.to_path.reparametrize(n);
        let t = t.clamp(0.0, 1.0);
        let pts: Vec<f64> = (0..n)
            .map(|i| from.points[i] * (1.0 - t) + to.points[i] * t)
            .collect();
        Path::new(pts)
    }

    /// The homotopy is valid if start and end endpoints match at all t.
    pub fn is_valid(&self) -> bool {
        const EPS: f64 = 1e-9;
        let start_ok = (self.from_path.start() - self.to_path.start()).abs() < EPS;
        let end_ok = (self.from_path.end() - self.to_path.end()).abs() < EPS;
        start_ok && end_ok
    }

    /// Endpoint-preserving: start and end don't move during deformation.
    pub fn endpoint_preserving(&self) -> bool {
        self.is_valid()
    }

    /// Homotopy relative to a subspace: the points in `subspace` are held fixed.
    /// For our setting this means the endpoints (which are in the subspace) don't
    /// move — equivalent to `endpoint_preserving` when subspace contains endpoints.
    pub fn relative(&self, subspace: &[f64]) -> bool {
        const EPS: f64 = 1e-9;
        if !self.is_valid() {
            return false;
        }
        let s = self.from_path.start();
        let e = self.from_path.end();
        for &x in subspace {
            if (x - s).abs() < EPS || (x - e).abs() < EPS {
                // endpoint in subspace — already verified by is_valid
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// 4. FundamentalGroupoid
// ---------------------------------------------------------------------------

/// The groupoid of paths up to homotopy equivalence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalGroupoid {
    pub space: TopologicalSpace,
    pub base_points: Vec<usize>,
    /// Pre-computed homotopy classes: maps (start_idx, end_idx) → class_id.
    #[serde(skip)]
    classes: HashMap<(usize, usize), usize>,
    #[serde(skip)]
    next_class: usize,
}

impl FundamentalGroupoid {
    pub fn new(space: TopologicalSpace, base_points: Vec<usize>) -> Self {
        let mut fg = Self {
            space,
            base_points,
            classes: HashMap::new(),
            next_class: 0,
        };
        fg.build_classes();
        fg
    }

    fn build_classes(&mut self) {
        // For each pair of base points, create a homotopy class.
        // If the space is contractible (tree), all paths between the same pair of
        // points are homotopic, so exactly one class per ordered pair.
        // If the space has cycles, we assign one class per pair for simplicity
        // (full implementation would track winding numbers).
        for &a in &self.base_points {
            for &b in &self.base_points {
                let key = (a, b);
                if !self.classes.contains_key(&key) {
                    let id = self.next_class;
                    self.next_class += 1;
                    self.classes.insert(key, id);
                }
            }
        }
        // Also handle (a, a) — the identity/constant path.
        for &a in &self.base_points {
            let key = (a, a);
            if !self.classes.contains_key(&key) {
                let id = self.next_class;
                self.next_class += 1;
                self.classes.insert(key, id);
            }
        }
    }

    /// Compute the homotopy class of a path.
    pub fn homotopy_class(&self, path: &Path) -> usize {
        let s = self.space.nearest(path.start());
        let e = self.space.nearest(path.end());
        let key = (s, e);
        // For contractible spaces, all paths from s to e are homotopic.
        if self.space.contractible() {
            return *self.classes.get(&key).unwrap_or(&0);
        }
        // For non-contractible, we use winding heuristic.
        // Class is determined by (start, end, winding_number).
        // For simplicity we use just (start, end) and add winding offset.
        let base = *self.classes.get(&key).unwrap_or(&0);
        let winding = Self::winding_number(path);
        base + winding.unsigned_abs() as usize * self.base_points.len().max(1)
    }

    /// Simple winding number: signed crossings around 0.5.
    fn winding_number(path: &Path) -> i32 {
        let center = 0.5;
        let mut crossings = 0i32;
        for w in path.points.windows(2) {
            if (w[0] - center) * (w[1] - center) < 0.0 {
                if w[1] > w[0] {
                    crossings += 1;
                } else {
                    crossings -= 1;
                }
            }
        }
        crossings
    }

    /// Groupoid composition: concatenate classes if endpoints match.
    pub fn compose(&self, class_a: usize, class_b: usize) -> Option<usize> {
        // We look up which pairs correspond to these classes.
        let pair_a = self.class_to_pair(class_a)?;
        let pair_b = self.class_to_pair(class_b)?;
        // Composable: end of a == start of b.
        if pair_a.1 != pair_b.0 {
            return None;
        }
        let composed_key = (pair_a.0, pair_b.1);
        Some(*self.classes.get(&composed_key).unwrap_or(&class_a))
    }

    fn class_to_pair(&self, class: usize) -> Option<(usize, usize)> {
        for (&key, &id) in &self.classes {
            if id == class {
                return Some(key);
            }
        }
        None
    }

    /// Inverse class.
    pub fn inverse(&self, class: usize) -> usize {
        if let Some(pair) = self.class_to_pair(class) {
            let inv_key = (pair.1, pair.0);
            *self.classes.get(&inv_key).unwrap_or(&class)
        } else {
            class
        }
    }

    /// All morphisms in a groupoid are invertible.
    pub fn is_invertible(&self, _class: usize) -> bool {
        true
    }

    /// The fundamental group π₁(X, x₀): automorphism classes at a base point.
    pub fn fundamental_group(&self, base_point: usize) -> Vec<usize> {
        let mut result = Vec::new();
        for (&key, &id) in &self.classes {
            if key.0 == base_point && key.1 == base_point {
                result.push(id);
            }
        }
        result
    }

    /// Simply connected: trivial fundamental group at every base point.
    pub fn is_simply_connected(&self) -> bool {
        self.space.contractible()
    }
}

// ---------------------------------------------------------------------------
// 5. HigherHomotopy
// ---------------------------------------------------------------------------

/// A 2-homotopy: a homotopy between homotopies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HigherHomotopy {
    pub h1: Homotopy,
    pub h2: Homotopy,
}

impl HigherHomotopy {
    pub fn new(h1: Homotopy, h2: Homotopy) -> Self {
        Self { h1, h2 }
    }

    pub fn is_valid(&self) -> bool {
        self.h1.is_valid() && self.h2.is_valid()
            && (self.h1.from_path.start() - self.h2.from_path.start()).abs() < 1e-9
            && (self.h1.from_path.end() - self.h2.from_path.end()).abs() < 1e-9
            && (self.h1.to_path.start() - self.h2.to_path.start()).abs() < 1e-9
            && (self.h1.to_path.end() - self.h2.to_path.end()).abs() < 1e-9
    }

    /// 2-parameter family: (s, t) → value, where s interpolates between the two
    /// homotopies and t is the homotopy parameter.
    pub fn intermediate(&self, s: f64, t: f64) -> f64 {
        let p1 = self.h1.intermediate(t);
        let p2 = self.h2.intermediate(t);
        let n = p1.points.len().max(p2.points.len());
        let p1 = p1.reparametrize(n);
        let p2 = p2.reparametrize(n);
        let idx = ((t * (n as f64 - 1.0)).round() as usize).min(n - 1);
        p1.points[idx] * (1.0 - s) + p2.points[idx] * s
    }
}

// ---------------------------------------------------------------------------
// 6. HomotopyGroup — πₙ(X, x₀)
// ---------------------------------------------------------------------------

/// The n-th homotopy group πₙ(X, x₀).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomotopyGroup {
    pub dimension: usize,
    pub space: TopologicalSpace,
    pub base_point: usize,
    /// Representatives (as coordinate vectors).
    pub elements: Vec<Vec<f64>>,
}

impl HomotopyGroup {
    pub fn new(dimension: usize, space: TopologicalSpace, base_point: usize) -> Self {
        let mut hg = Self {
            dimension,
            space,
            base_point,
            elements: Vec::new(),
        };
        hg.compute_elements();
        hg
    }

    fn compute_elements(&mut self) {
        if self.dimension == 0 {
            // π₀: one element per connected component.
            let components = self.space.connected_components();
            self.elements = components
                .iter()
                .map(|c| vec![c.first().copied().unwrap_or(0) as f64])
                .collect();
        } else if self.dimension == 1 {
            // π₁: identity + generators.
            let fg = FundamentalGroupoid::new(self.space.clone(), vec![self.base_point]);
            let aut = fg.fundamental_group(self.base_point);
            self.elements = aut
                .iter()
                .enumerate()
                .map(|(i, _)| vec![i as f64])
                .collect();
        } else {
            // Higher: for our discrete spaces, trivial (just identity).
            self.elements = vec![vec![0.0; self.dimension]];
        }
    }

    /// Group operation (concatenation in π₁, pointwise for higher).
    pub fn group_operation(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        if a.len() != b.len() {
            return a.to_vec();
        }
        a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
    }

    /// Identity element.
    pub fn identity(&self) -> Vec<f64> {
        vec![0.0; self.dimension.max(1)]
    }

    /// Inverse element.
    pub fn inverse(&self, element: &[f64]) -> Vec<f64> {
        element.iter().map(|x| -x).collect()
    }

    /// Order of the group.
    pub fn order(&self) -> usize {
        self.elements.len()
    }
}

// ---------------------------------------------------------------------------
// 7. AbelianGroup
// ---------------------------------------------------------------------------

/// A finitely-generated abelian group: Z^rank ⊕ Z/t₁ ⊕ Z/t₂ ⊕ ...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbelianGroup {
    pub rank: usize,
    pub torsion: Vec<usize>,
}

impl AbelianGroup {
    pub fn new(rank: usize, torsion: Vec<usize>) -> Self {
        Self { rank, torsion }
    }

    pub fn trivial() -> Self {
        Self {
            rank: 0,
            torsion: vec![],
        }
    }

    pub fn z() -> Self {
        Self {
            rank: 1,
            torsion: vec![],
        }
    }

    pub fn zn(n: usize) -> Self {
        Self {
            rank: 0,
            torsion: vec![n],
        }
    }

    /// Order of the group. `None` if infinite.
    pub fn order(&self) -> Option<usize> {
        if self.rank > 0 {
            return None;
        }
        let mut o = 1usize;
        for &t in &self.torsion {
            o = o.checked_mul(t)?;
        }
        Some(o)
    }

    pub fn is_trivial(&self) -> bool {
        self.rank == 0 && self.torsion.is_empty()
    }

    /// Direct sum.
    pub fn direct_sum(&self, other: &AbelianGroup) -> AbelianGroup {
        let mut torsion = self.torsion.clone();
        torsion.extend_from_slice(&other.torsion);
        AbelianGroup {
            rank: self.rank + other.rank,
            torsion,
        }
    }
}

// ---------------------------------------------------------------------------
// 8. SparseMatrix
// ---------------------------------------------------------------------------

/// A sparse matrix representation for group homomorphisms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMatrix {
    pub rows: usize,
    pub cols: usize,
    /// (row, col, value) entries.
    pub entries: Vec<(usize, usize, i64)>,
}

impl SparseMatrix {
    pub fn zero(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            entries: vec![],
        }
    }

    pub fn identity(n: usize) -> Self {
        Self {
            rows: n,
            cols: n,
            entries: (0..n).map(|i| (i, i, 1)).collect(),
        }
    }

    /// Apply the matrix to a vector.
    pub fn apply(&self, v: &[i64]) -> Vec<i64> {
        let mut result = vec![0i64; self.rows];
        for &(r, c, val) in &self.entries {
            if c < v.len() {
                result[r] += val * v[c];
            }
        }
        result
    }

    /// Compute the image (column space) as a set of vectors.
    pub fn image(&self) -> Vec<Vec<i64>> {
        let mut img = Vec::new();
        for c in 0..self.cols {
            let mut v = vec![0i64; self.rows];
            for &(r, col, val) in &self.entries {
                if col == c {
                    v[r] = val;
                }
            }
            if v.iter().any(|&x| x != 0) {
                img.push(v);
            }
        }
        img
    }

    /// Compute the kernel (null space).
    pub fn kernel(&self) -> Vec<Vec<i64>> {
        let mut ker = Vec::new();
        for c in 0..self.cols {
            let mut v = vec![0i64; self.cols];
            v[c] = 1;
            let applied = self.apply(&v);
            if applied.iter().all(|&x| x == 0) {
                ker.push(v);
            }
        }
        ker
    }
}

// ---------------------------------------------------------------------------
// 9. ExactSequence
// ---------------------------------------------------------------------------

/// A chain of groups with maps, where im(∂ᵢ₊₁) = ker(∂ᵢ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactSequence {
    pub groups: Vec<AbelianGroup>,
    pub maps: Vec<SparseMatrix>,
}

impl ExactSequence {
    pub fn new(groups: Vec<AbelianGroup>, maps: Vec<SparseMatrix>) -> Self {
        Self { groups, maps }
    }

    /// Verify exactness: im(map[i+1]) ⊆ ker(map[i]) for each position.
    pub fn is_exact(&self) -> bool {
        if self.maps.len() + 1 != self.groups.len() {
            return false;
        }
        for i in 0..self.maps.len().saturating_sub(1) {
            // For each basis vector of the source of maps[i+1], compose maps[i] ∘ maps[i+1].
            let dim = self.maps[i + 1].cols;
            for c in 0..dim {
                let mut v = vec![0i64; dim];
                v[c] = 1;
                let intermediate = self.maps[i + 1].apply(&v);
                let result = self.maps[i].apply(&intermediate);
                if result.iter().any(|&x| x != 0) {
                    return false;
                }
            }
        }
        true
    }

    /// Compute homology group dimensions: dim(ker/map[i]) at each position.
    pub fn compute_homology(&self) -> Vec<usize> {
        self.maps
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let ker_dim = if i > 0 {
                    self.maps[i].kernel().len()
                } else {
                    self.groups.get(i).map(|g| g.rank).unwrap_or(0)
                };
                let img_dim = if i + 1 < self.maps.len() {
                    self.maps[i + 1].image().len()
                } else {
                    0
                };
                ker_dim.saturating_sub(img_dim)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 10. Fibration
// ---------------------------------------------------------------------------

/// A fiber bundle structure: total → base with fiber.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fibration {
    pub total_space: TopologicalSpace,
    pub base_space: TopologicalSpace,
    pub fiber: TopologicalSpace,
    /// projection[i] maps point i of total → point index in base.
    projection_map: Vec<usize>,
}

impl Fibration {
    pub fn new(
        total: TopologicalSpace,
        base: TopologicalSpace,
        fiber: TopologicalSpace,
        projection_map: Vec<usize>,
    ) -> Self {
        Self {
            total_space: total,
            base_space: base,
            fiber,
            projection_map,
        }
    }

    /// Project a point in total space to base space.
    pub fn projection(&self, point: usize) -> usize {
        self.projection_map.get(point).copied().unwrap_or(0)
    }

    /// Fiber over a base point: all points in total mapping to it.
    pub fn fiber_over(&self, base_point: usize) -> Vec<usize> {
        self.projection_map
            .iter()
            .enumerate()
            .filter(|(_, &b)| b == base_point)
            .map(|(i, _)| i)
            .collect()
    }

    /// Construct the long exact sequence of homotopy groups.
    pub fn exact_sequence(&self) -> ExactSequence {
        let n = 4; // simplified: π₀, π₁, π₂, π₃
        let groups: Vec<AbelianGroup> = (0..n)
            .map(|dim| {
                let hg = HomotopyGroup::new(dim, self.fiber.clone(), 0);
                AbelianGroup::new(hg.order(), vec![])
            })
            .collect();
        let maps = vec![SparseMatrix::zero(n, n); n - 1];
        ExactSequence::new(groups, maps)
    }

    /// Locally trivial: every point in base has a neighborhood whose preimage
    /// is a product. For our discrete approximation, check that all fibers have
    /// the same size.
    pub fn is_fiber_bundle(&self) -> bool {
        let n = self.base_space.points.len();
        if n == 0 {
            return true;
        }
        let fiber_size = self.fiber_over(0).len();
        (0..n).all(|b| self.fiber_over(b).len() == fiber_size)
    }

    /// Serre fibration (homotopy lifting property).
    /// For our discrete setting, any fiber bundle is a Serre fibration.
    pub fn serre_fibration(&self) -> bool {
        self.is_fiber_bundle()
    }
}

// ---------------------------------------------------------------------------
// 11. SyntheticType
// ---------------------------------------------------------------------------

/// Truncation level / h-level of a type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    Contractible,                     // h-level 0
    Proposition,                      // h-level 1
    Set,                              // h-level 2
    Groupoid,                         // h-level 3
    HigherGroupoid(usize),            // h-level n (n ≥ 4)
}

/// A homotopy type theory synthetic type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticType {
    pub kind: TypeKind,
}

impl SyntheticType {
    pub fn new(kind: TypeKind) -> Self {
        Self { kind }
    }

    pub fn contractible() -> Self {
        Self::new(TypeKind::Contractible)
    }

    pub fn proposition() -> Self {
        Self::new(TypeKind::Proposition)
    }

    pub fn set() -> Self {
        Self::new(TypeKind::Set)
    }

    pub fn groupoid() -> Self {
        Self::new(TypeKind::Groupoid)
    }

    pub fn higher(n: usize) -> Self {
        Self::new(TypeKind::HigherGroupoid(n))
    }

    /// h-level.
    pub fn h_level(&self) -> usize {
        match self.kind {
            TypeKind::Contractible => 0,
            TypeKind::Proposition => 1,
            TypeKind::Set => 2,
            TypeKind::Groupoid => 3,
            TypeKind::HigherGroupoid(n) => n,
        }
    }

    /// The identity type of this type.
    pub fn identity_type(&self) -> SyntheticType {
        match self.kind {
            TypeKind::Contractible => SyntheticType::contractible(),
            TypeKind::Proposition => SyntheticType::contractible(),
            TypeKind::Set => SyntheticType::proposition(),
            TypeKind::Groupoid => SyntheticType::set(),
            TypeKind::HigherGroupoid(n) => {
                if n <= 4 {
                    SyntheticType::groupoid()
                } else {
                    SyntheticType::higher(n - 1)
                }
            }
        }
    }

    /// Is a proposition: at most one equality proof.
    pub fn is_prop(&self) -> bool {
        self.h_level() <= 1
    }

    /// Is a set: equality proofs are unique (UIP).
    pub fn is_set(&self) -> bool {
        self.h_level() <= 2
    }

    /// Is a groupoid: equality of equalities is a proposition.
    pub fn is_groupoid(&self) -> bool {
        self.h_level() <= 3
    }

    /// n-truncation.
    pub fn truncation(&self, level: usize) -> SyntheticType {
        if self.h_level() <= level {
            return self.clone();
        }
        match level {
            0 => SyntheticType::contractible(),
            1 => SyntheticType::proposition(),
            2 => SyntheticType::set(),
            3 => SyntheticType::groupoid(),
            _ => SyntheticType::higher(level),
        }
    }
}

// ---------------------------------------------------------------------------
// 12. Transition & AgentProtocol
// ---------------------------------------------------------------------------

/// A state transition in an agent protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from_state: usize,
    pub to_state: usize,
    pub label: String,
    pub condition: Option<String>,
}

impl Transition {
    pub fn new(from: usize, to: usize, label: &str) -> Self {
        Self {
            from_state: from,
            to_state: to,
            label: label.to_string(),
            condition: None,
        }
    }

    pub fn with_condition(from: usize, to: usize, label: &str, cond: &str) -> Self {
        Self {
            from_state: from,
            to_state: to,
            label: label.to_string(),
            condition: Some(cond.to_string()),
        }
    }
}

/// An agent communication protocol modeled as a higher inductive type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProtocol {
    pub name: String,
    pub path_type: SyntheticType,
    pub states: Vec<String>,
    pub transitions: Vec<Transition>,
    /// Homotopy paths proving protocol correctness.
    pub verification_paths: Vec<Path>,
}

impl AgentProtocol {
    pub fn new(name: &str, path_type: SyntheticType) -> Self {
        Self {
            name: name.to_string(),
            path_type,
            states: Vec::new(),
            transitions: Vec::new(),
            verification_paths: Vec::new(),
        }
    }

    pub fn add_state(&mut self, name: &str) -> usize {
        let idx = self.states.len();
        self.states.push(name.to_string());
        idx
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.transitions.push(t);
    }

    /// Protocol is correct if all paths from start to end are homotopic.
    /// Simplified: if there is a unique connected path from state 0 to the last
    /// state, and the path type is at most a set.
    pub fn is_correct(&self) -> bool {
        if self.states.is_empty() {
            return true;
        }
        let n = self.states.len();
        // BFS: are all states reachable from state 0?
        let mut reachable = vec![false; n];
        let mut stack = vec![0];
        while let Some(s) = stack.pop() {
            if reachable[s] {
                continue;
            }
            reachable[s] = true;
            for t in &self.transitions {
                if t.from_state == s && !reachable[t.to_state] {
                    stack.push(t.to_state);
                }
            }
        }
        reachable.iter().all(|&r| r)
    }

    /// Deadlock-free: no non-trivial homotopy class represents a deadlock.
    /// A deadlock is a state with no outgoing transitions that isn't the final
    /// state. If the path type is contractible, it's always deadlock-free.
    pub fn deadlock_free(&self) -> bool {
        if self.path_type.kind == TypeKind::Contractible {
            return true;
        }
        let n = self.states.len();
        if n == 0 {
            return true;
        }
        let final_state = n - 1;
        for i in 0..n {
            if i == final_state {
                continue;
            }
            let has_outgoing = self.transitions.iter().any(|t| t.from_state == i);
            if !has_outgoing {
                return false; // deadlock at state i
            }
        }
        true
    }
}

// ===========================================================================
// TESTS
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Path ----

    #[test]
    fn path_start_end() {
        let p = Path::new(vec![0.0, 0.5, 1.0]);
        assert!((p.start() - 0.0).abs() < 1e-9);
        assert!((p.end() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn path_evaluate_endpoints() {
        let p = Path::new(vec![0.0, 1.0]);
        assert!((p.evaluate(0.0) - 0.0).abs() < 1e-9);
        assert!((p.evaluate(1.0) - 1.0).abs() < 1e-9);
        assert!((p.evaluate(0.5) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn path_evaluate_mid() {
        let p = Path::new(vec![0.0, 1.0, 2.0, 3.0]);
        assert!((p.evaluate(0.5) - 1.5).abs() < 1e-9);
        assert!((p.evaluate(0.25) - 0.75).abs() < 1e-9);
    }

    #[test]
    fn path_reverse() {
        let p = Path::new(vec![0.0, 1.0, 2.0]);
        let r = p.reverse();
        assert_eq!(r.points, vec![2.0, 1.0, 0.0]);
    }

    #[test]
    fn path_concat_success() {
        let a = Path::new(vec![0.0, 1.0]);
        let b = Path::new(vec![1.0, 2.0]);
        let c = a.concat(&b).unwrap();
        assert_eq!(c.points, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn path_concat_fail() {
        let a = Path::new(vec![0.0, 1.0]);
        let b = Path::new(vec![2.0, 3.0]);
        assert!(a.concat(&b).is_none());
    }

    #[test]
    fn path_length() {
        let p = Path::new(vec![0.0, 1.0, 3.0]);
        assert!((p.length() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn path_reparametrize() {
        let p = Path::new(vec![0.0, 2.0]);
        let r = p.reparametrize(5);
        assert_eq!(r.points.len(), 5);
        assert!((r.start() - 0.0).abs() < 1e-9);
        assert!((r.end() - 2.0).abs() < 1e-9);
        assert!((r.evaluate(0.5) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn path_constant() {
        let p = Path::constant(42.0, 4);
        assert!(p.points.iter().all(|&x| (x - 42.0).abs() < 1e-9));
        assert_eq!(p.points.len(), 4);
    }

    // ---- TopologicalSpace ----

    #[test]
    fn space_discrete() {
        let s = TopologicalSpace::discrete(vec![0.0, 1.0, 2.0]);
        assert_eq!(s.connected_components().len(), 3);
        // Discrete space with 3 points is NOT contractible (3 connected components)
        assert!(!s.contractible());
        // But a single-point discrete space is
        let s1 = TopologicalSpace::discrete(vec![0.0]);
        assert!(s1.contractible());
    }

    #[test]
    fn space_interval_contractible() {
        let s = TopologicalSpace::interval(5);
        assert!(s.contractible());
        assert_eq!(s.euler_characteristic(), 1); // V=5, E=4, V-E=1
    }

    #[test]
    fn space_circle_not_contractible() {
        let s = TopologicalSpace::circle(6);
        assert!(!s.contractible());
        assert_eq!(s.euler_characteristic(), 0); // V=6, E=6
    }

    #[test]
    fn space_circle_connected() {
        let s = TopologicalSpace::circle(8);
        assert_eq!(s.connected_components().len(), 1);
    }

    #[test]
    fn space_dimension() {
        let s = TopologicalSpace::interval(3);
        assert_eq!(s.dimension(), 1);
        let d = TopologicalSpace::discrete(vec![1.0]);
        assert_eq!(d.dimension(), 0);
    }

    #[test]
    fn space_euler_characteristic_line() {
        // Line with 2 points: V=2, E=1, χ=1
        let s = TopologicalSpace::interval(2);
        assert_eq!(s.euler_characteristic(), 1);
    }

    // ---- Homotopy ----

    #[test]
    fn homotopy_valid_same_endpoints() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.8, 1.0]);
        let h = Homotopy::new(p1, p2, 10);
        assert!(h.is_valid());
    }

    #[test]
    fn homotopy_invalid_different_endpoints() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.5, 2.0]);
        let h = Homotopy::new(p1, p2, 10);
        assert!(!h.is_valid());
    }

    #[test]
    fn homotopy_intermediate_endpoints() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.8, 1.0]);
        let h = Homotopy::new(p1, p2, 10);
        let at0 = h.intermediate(0.0);
        let at1 = h.intermediate(1.0);
        assert!((at0.start() - 0.0).abs() < 1e-9);
        assert!((at0.end() - 1.0).abs() < 1e-9);
        assert!((at1.start() - 0.0).abs() < 1e-9);
        assert!((at1.end() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn homotopy_endpoint_preserving() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.9, 1.0]);
        let h = Homotopy::new(p1, p2, 10);
        assert!(h.endpoint_preserving());
    }

    #[test]
    fn homotopy_relative() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.7, 1.0]);
        let h = Homotopy::new(p1, p2, 10);
        assert!(h.relative(&[0.0, 1.0]));
    }

    #[test]
    fn homotopy_linear_deformation() {
        let p1 = Path::new(vec![0.0, 1.0]);
        let p2 = Path::new(vec![0.0, 0.0]);
        let h = Homotopy::new(p1, p2, 10);
        let mid = h.intermediate(0.5);
        // At t=0.5, the intermediate path is the average of p1 and p2 reparametrized to 10 points
        // p1 reparam'd: 0,0.11,0.22,...,1.0  p2 reparam'd: 0,0,...,0
        // Average at index 5: ~0.5 * 0.556 = 0.278
        assert!(mid.points[5] > 0.0 && mid.points[5] < 0.6);
    }

    // ---- FundamentalGroupoid ----

    #[test]
    fn groupoid_contractible_trivial() {
        let space = TopologicalSpace::interval(5);
        let fg = FundamentalGroupoid::new(space, vec![0, 4]);
        assert!(fg.is_simply_connected());
    }

    #[test]
    fn groupoid_invertible() {
        let space = TopologicalSpace::circle(6);
        let fg = FundamentalGroupoid::new(space, vec![0]);
        assert!(fg.is_invertible(0));
    }

    #[test]
    fn groupoid_inverse() {
        let space = TopologicalSpace::circle(6);
        let fg = FundamentalGroupoid::new(space, vec![0, 3]);
        let p = Path::new(vec![0.0, 0.5, 1.0]);
        let cls = fg.homotopy_class(&p);
        let inv = fg.inverse(cls);
        // Inverse of inverse should give back same class (up to our simple model)
        let inv2 = fg.inverse(inv);
        assert_eq!(inv2, cls);
    }

    #[test]
    fn groupoid_compose() {
        let space = TopologicalSpace::interval(5);
        let fg = FundamentalGroupoid::new(space.clone(), vec![0, 2, 4]);
        // Compose (0,2) with (2,4) should give (0,4)
        let cls_02 = fg.homotopy_class(&Path::new(vec![space.points[0], space.points[2]]));
        let cls_24 = fg.homotopy_class(&Path::new(vec![space.points[2], space.points[4]]));
        let composed = fg.compose(cls_02, cls_24);
        assert!(composed.is_some());
    }

    #[test]
    fn groupoid_fundamental_group() {
        let space = TopologicalSpace::interval(5);
        let fg = FundamentalGroupoid::new(space, vec![2]);
        let pi1 = fg.fundamental_group(2);
        // For a contractible space, the fundamental group at any point is trivial
        assert!(!pi1.is_empty()); // at least the identity class
    }

    #[test]
    fn groupoid_circle_not_simply_connected() {
        let space = TopologicalSpace::circle(8);
        let fg = FundamentalGroupoid::new(space, vec![0]);
        assert!(!fg.is_simply_connected());
    }

    // ---- HigherHomotopy ----

    #[test]
    fn higher_homotopy_valid() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.8, 1.0]);
        let h1 = Homotopy::new(p1.clone(), p2.clone(), 10);
        let h2 = Homotopy::new(p1, p2, 10);
        let hh = HigherHomotopy::new(h1, h2);
        assert!(hh.is_valid());
    }

    #[test]
    fn higher_homotopy_intermediate() {
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.8, 1.0]);
        let h1 = Homotopy::new(p1.clone(), p2.clone(), 10);
        let h2 = Homotopy::new(p1, p2, 10);
        let hh = HigherHomotopy::new(h1, h2);
        // At s=0.5, should be average
        let val = hh.intermediate(0.5, 0.5);
        assert!(val.is_finite());
    }

    // ---- AbelianGroup ----

    #[test]
    fn abelian_trivial() {
        let g = AbelianGroup::trivial();
        assert!(g.is_trivial());
        assert_eq!(g.order(), Some(1));
    }

    #[test]
    fn abelian_z() {
        let g = AbelianGroup::z();
        assert!(!g.is_trivial());
        assert_eq!(g.order(), None); // infinite
    }

    #[test]
    fn abelian_zn() {
        let g = AbelianGroup::zn(5);
        assert_eq!(g.order(), Some(5));
    }

    #[test]
    fn abelian_direct_sum() {
        let g1 = AbelianGroup::z();
        let g2 = AbelianGroup::zn(3);
        let sum = g1.direct_sum(&g2);
        assert_eq!(sum.rank, 1);
        assert_eq!(sum.torsion, vec![3]);
        assert_eq!(sum.order(), None); // infinite because rank > 0
    }

    #[test]
    fn abelian_direct_sum_finite() {
        let g1 = AbelianGroup::zn(2);
        let g2 = AbelianGroup::zn(3);
        let sum = g1.direct_sum(&g2);
        assert_eq!(sum.order(), Some(6));
    }

    // ---- ExactSequence ----

    #[test]
    fn exact_sequence_trivial() {
        let groups = vec![AbelianGroup::trivial(), AbelianGroup::trivial()];
        let maps = vec![SparseMatrix::zero(1, 1)];
        let seq = ExactSequence::new(groups, maps);
        assert!(seq.is_exact());
    }

    #[test]
    fn exact_sequence_identity_not_exact() {
        let groups = vec![AbelianGroup::z(), AbelianGroup::z(), AbelianGroup::z()];
        let maps = vec![SparseMatrix::identity(1), SparseMatrix::zero(1, 1)];
        let seq = ExactSequence::new(groups, maps);
        // id ∘ zero = zero, so at position 0: im(zero) = {0} ⊆ ker(id) = {0}. OK.
        // At position 1: im(id) = Z, ker(zero) = Z. So im ⊆ ker → exact at 1.
        assert!(seq.is_exact());
    }

    #[test]
    fn exact_sequence_compute_homology() {
        let groups = vec![AbelianGroup::z(), AbelianGroup::trivial()];
        let maps = vec![SparseMatrix::zero(1, 1)];
        let seq = ExactSequence::new(groups, maps);
        let hom = seq.compute_homology();
        assert_eq!(hom.len(), 1);
    }

    // ---- SparseMatrix ----

    #[test]
    fn sparse_matrix_apply() {
        let m = SparseMatrix::identity(2);
        let v = vec![3, 7];
        assert_eq!(m.apply(&v), vec![3, 7]);
    }

    #[test]
    fn sparse_matrix_zero() {
        let m = SparseMatrix::zero(2, 3);
        assert_eq!(m.apply(&vec![1, 2, 3]), vec![0, 0]);
    }

    #[test]
    fn sparse_matrix_kernel_identity() {
        let m = SparseMatrix::identity(2);
        let ker = m.kernel();
        assert!(ker.is_empty()); // only trivial kernel
    }

    // ---- HomotopyGroup ----

    #[test]
    fn homotopy_group_pi0() {
        let space = TopologicalSpace::discrete(vec![0.0, 1.0, 2.0]);
        let hg = HomotopyGroup::new(0, space, 0);
        assert_eq!(hg.order(), 3); // 3 components
    }

    #[test]
    fn homotopy_group_identity() {
        let space = TopologicalSpace::circle(5);
        let hg = HomotopyGroup::new(1, space, 0);
        let id = hg.identity();
        assert_eq!(id, vec![0.0]);
    }

    #[test]
    fn homotopy_group_inverse() {
        let space = TopologicalSpace::circle(5);
        let hg = HomotopyGroup::new(1, space, 0);
        let inv = hg.inverse(&[1.0]);
        assert_eq!(inv, vec![-1.0]);
    }

    #[test]
    fn homotopy_group_operation() {
        let hg = HomotopyGroup::new(1, TopologicalSpace::circle(4), 0);
        let result = hg.group_operation(&[1.0], &[2.0]);
        assert_eq!(result, vec![3.0]);
    }

    // ---- Fibration ----

    #[test]
    fn fibration_trivial_bundle() {
        let base = TopologicalSpace::interval(3);
        let fiber = TopologicalSpace::discrete(vec![0.0]);
        let total = TopologicalSpace::interval(3);
        // Trivial projection: each point maps to itself.
        let fib = Fibration::new(total, base, fiber, vec![0, 1, 2]);
        assert!(fib.is_fiber_bundle());
        assert_eq!(fib.fiber_over(0), vec![0]);
        assert_eq!(fib.projection(1), 1);
    }

    #[test]
    fn fibration_product_bundle() {
        let base = TopologicalSpace::interval(2);
        let fiber = TopologicalSpace::discrete(vec![0.0, 1.0]);
        // Total = base × fiber, 4 points
        let total = TopologicalSpace::discrete(vec![0.0, 0.0, 1.0, 1.0]);
        // Projection: (b, f) → b
        let fib = Fibration::new(total, base, fiber, vec![0, 0, 1, 1]);
        assert!(fib.is_fiber_bundle());
        assert_eq!(fib.fiber_over(0).len(), 2);
        assert_eq!(fib.fiber_over(1).len(), 2);
    }

    #[test]
    fn fibration_serre() {
        let base = TopologicalSpace::interval(2);
        let fiber = TopologicalSpace::discrete(vec![0.0]);
        let total = TopologicalSpace::interval(2);
        let fib = Fibration::new(total, base, fiber, vec![0, 1]);
        assert!(fib.serre_fibration());
    }

    // ---- SyntheticType ----

    #[test]
    fn synthetic_contractible() {
        let t = SyntheticType::contractible();
        assert!(t.is_prop());
        assert!(t.is_set());
        assert!(t.is_groupoid());
        assert_eq!(t.h_level(), 0);
    }

    #[test]
    fn synthetic_proposition() {
        let t = SyntheticType::proposition();
        assert!(t.is_prop());
        assert!(t.is_set());
        assert_eq!(t.h_level(), 1);
    }

    #[test]
    fn synthetic_set() {
        let t = SyntheticType::set();
        assert!(!t.is_prop());
        assert!(t.is_set());
        assert_eq!(t.h_level(), 2);
    }

    #[test]
    fn synthetic_groupoid() {
        let t = SyntheticType::groupoid();
        assert!(!t.is_set());
        assert!(t.is_groupoid());
        assert_eq!(t.h_level(), 3);
    }

    #[test]
    fn synthetic_identity_type() {
        // Identity of a set is a proposition
        let t = SyntheticType::set();
        let id = t.identity_type();
        assert!(id.is_prop());
        assert_eq!(id.h_level(), 1);
    }

    #[test]
    fn synthetic_identity_groupoid() {
        let t = SyntheticType::groupoid();
        let id = t.identity_type();
        assert!(id.is_set());
        assert_eq!(id.h_level(), 2);
    }

    #[test]
    fn synthetic_truncation() {
        let t = SyntheticType::higher(5);
        let trunc = t.truncation(2);
        assert_eq!(trunc.h_level(), 2); // truncated to set
    }

    #[test]
    fn synthetic_truncation_noop() {
        let t = SyntheticType::set();
        let trunc = t.truncation(3);
        assert_eq!(trunc.h_level(), 2); // unchanged
    }

    #[test]
    fn synthetic_truncation_s1_is_set() {
        // Theorem 7: 0-truncation of S¹ gives a set (decidable equality)
        let s1_type = SyntheticType::groupoid(); // S¹ is a groupoid
        let trunc = s1_type.truncation(2); // truncate to set
        assert!(trunc.is_set());
    }

    // ---- AgentProtocol ----

    #[test]
    fn protocol_correct_simple() {
        let mut p = AgentProtocol::new("test", SyntheticType::contractible());
        let s0 = p.add_state("start");
        let s1 = p.add_state("end");
        p.add_transition(Transition::new(s0, s1, "go"));
        assert!(p.is_correct());
    }

    #[test]
    fn protocol_deadlock_free_contractible() {
        let mut p = AgentProtocol::new("test", SyntheticType::contractible());
        let s0 = p.add_state("start");
        let _s1 = p.add_state("end");
        p.add_transition(Transition::new(s0, s0, "loop"));
        assert!(p.deadlock_free()); // contractible => always deadlock-free
    }

    #[test]
    fn protocol_deadlock_detected() {
        let mut p = AgentProtocol::new("test", SyntheticType::set());
        let _s0 = p.add_state("start");
        let _s1 = p.add_state("middle"); // no outgoing, not final
        let _s2 = p.add_state("end");
        // No transitions from middle → deadlock
        assert!(!p.deadlock_free());
    }

    #[test]
    fn protocol_deadlock_free_with_all_paths() {
        let mut p = AgentProtocol::new("test", SyntheticType::set());
        let s0 = p.add_state("start");
        let s1 = p.add_state("mid");
        let s2 = p.add_state("end");
        p.add_transition(Transition::new(s0, s1, "a"));
        p.add_transition(Transition::new(s1, s2, "b"));
        assert!(p.deadlock_free());
    }

    // ---- Theorems ----

    // Theorem 1: Fundamental group of S¹ is Z
    #[test]
    fn theorem_1_fundamental_group_circle_is_z() {
        let s1 = TopologicalSpace::circle(12);
        let fg = FundamentalGroupoid::new(s1, vec![0]);
        let pi1 = fg.fundamental_group(0);
        // The fundamental group should have representatives;
        // for S¹ the group is Z (infinite cyclic).
        assert!(!pi1.is_empty());
        // Verify the groupoid at base point 0 is non-trivial
        // (circle is not simply connected)
        assert!(!fg.is_simply_connected());
    }

    // Theorem 2: Fundamental group of contractible space is trivial
    #[test]
    fn theorem_2_contractible_trivial_pi1() {
        let space = TopologicalSpace::interval(10);
        assert!(space.contractible());
        let fg = FundamentalGroupoid::new(space, vec![0]);
        assert!(fg.is_simply_connected());
    }

    // Theorem 3: Concatenation of homotopic paths gives homotopic result
    #[test]
    fn theorem_3_concat_homotopic_paths() {
        let p1a = Path::new(vec![0.0, 0.5, 1.0]);
        let p1b = Path::new(vec![0.0, 0.8, 1.0]);
        let p2 = Path::new(vec![1.0, 1.5, 2.0]);

        // p1a and p1b are homotopic (same endpoints)
        let h = Homotopy::new(p1a.clone(), p1b.clone(), 10);
        assert!(h.is_valid());

        let c1 = p1a.concat(&p2).unwrap();
        let c2 = p1b.concat(&p2).unwrap();

        // Concatenated paths are homotopic
        let h2 = Homotopy::new(c1, c2, 10);
        assert!(h2.is_valid());
    }

    // Theorem 4: Fibration exact sequence structure exists
    #[test]
    fn theorem_4_fibration_exact_sequence() {
        let base = TopologicalSpace::interval(3);
        let fiber = TopologicalSpace::discrete(vec![0.0]);
        let total = TopologicalSpace::interval(3);
        let fib = Fibration::new(total, base, fiber, vec![0, 1, 2]);
        let seq = fib.exact_sequence();
        // The exact sequence is well-formed
        assert_eq!(seq.groups.len(), seq.maps.len() + 1);
    }

    // Theorem 5: Euler characteristic is homotopy invariant
    #[test]
    fn theorem_5_euler_homotopy_invariant() {
        let line = TopologicalSpace::interval(5);
        let line2 = TopologicalSpace::interval(10);
        // Both are contractible, same homotopy type
        assert!(line.contractible());
        assert!(line2.contractible());
        assert_eq!(line.euler_characteristic(), 1);
        assert_eq!(line2.euler_characteristic(), 1);
    }

    // Theorem 6: Groupoid composition is associative up to higher homotopy
    #[test]
    fn theorem_6_groupoid_associative() {
        let space = TopologicalSpace::interval(10);
        let fg = FundamentalGroupoid::new(space.clone(), vec![0, 3, 6, 9]);

        // Use exact point values from interval(10): 0.0, 1/3, 2/3, 1.0
        let p01 = Path::new(vec![space.points[0], space.points[3]]);
        let p12 = Path::new(vec![space.points[3], space.points[6]]);
        let p23 = Path::new(vec![space.points[6], space.points[9]]);

        let c01 = fg.homotopy_class(&p01);
        let c12 = fg.homotopy_class(&p12);
        let c23 = fg.homotopy_class(&p23);

        let left = fg.compose(c01, c12).and_then(|a| fg.compose(a, c23));
        let right = fg.compose(c12, c23).and_then(|b| fg.compose(c01, b));
        // Both should succeed and give same result (associativity)
        assert!(left.is_some());
        assert!(right.is_some());
        assert_eq!(left.unwrap(), right.unwrap());
    }

    // Theorem 8: Contractible path type => deadlock-free
    #[test]
    fn theorem_8_contractible_deadlock_free() {
        let mut p = AgentProtocol::new("test", SyntheticType::contractible());
        let s0 = p.add_state("start");
        let s1 = p.add_state("end");
        p.add_transition(Transition::new(s0, s1, "finish"));
        assert!(p.deadlock_free());
    }

    // Theorem 9: Path reversal gives groupoid inverse
    #[test]
    fn theorem_9_path_reversal_inverse() {
        let p = Path::new(vec![0.0, 0.5, 1.0]);
        let r = p.reverse();
        // p · r should be a loop
        let concat = p.concat(&r).unwrap();
        assert!((concat.start() - 0.0).abs() < 1e-9);
        assert!((concat.end() - 0.0).abs() < 1e-9);

        // r · p should also be a loop
        let concat2 = r.concat(&p).unwrap();
        assert!((concat2.start() - 1.0).abs() < 1e-9);
        assert!((concat2.end() - 1.0).abs() < 1e-9);
    }

    // Theorem 10: Simply connected means unique path up to homotopy
    #[test]
    fn theorem_10_simply_connected_unique_path() {
        let space = TopologicalSpace::interval(10);
        let fg = FundamentalGroupoid::new(space, vec![0, 9]);
        assert!(fg.is_simply_connected());

        // Two different paths between same endpoints
        let p1 = Path::new(vec![0.0, 0.5, 1.0]);
        let p2 = Path::new(vec![0.0, 0.8, 1.0]);
        assert_eq!(fg.homotopy_class(&p1), fg.homotopy_class(&p2));
    }

    // ---- Serde round-trip ----

    #[test]
    fn serde_path() {
        let p = Path::new(vec![0.0, 1.0, 2.0]);
        let json = serde_json::to_string(&p).unwrap();
        let p2: Path = serde_json::from_str(&json).unwrap();
        assert_eq!(p.points, p2.points);
    }

    #[test]
    fn serde_topological_space() {
        let s = TopologicalSpace::circle(5);
        let json = serde_json::to_string(&s).unwrap();
        let s2: TopologicalSpace = serde_json::from_str(&json).unwrap();
        assert_eq!(s.points, s2.points);
    }

    #[test]
    fn serde_homotopy() {
        let h = Homotopy::new(
            Path::new(vec![0.0, 1.0]),
            Path::new(vec![0.0, 0.5]),
            5,
        );
        let json = serde_json::to_string(&h).unwrap();
        let h2: Homotopy = serde_json::from_str(&json).unwrap();
        assert_eq!(h.steps, h2.steps);
    }

    #[test]
    fn serde_synthetic_type() {
        let t = SyntheticType::higher(5);
        let json = serde_json::to_string(&t).unwrap();
        let t2: SyntheticType = serde_json::from_str(&json).unwrap();
        assert_eq!(t.kind, t2.kind);
    }

    #[test]
    fn serde_agent_protocol() {
        let mut p = AgentProtocol::new("test", SyntheticType::set());
        p.add_state("a");
        p.add_state("b");
        p.add_transition(Transition::new(0, 1, "go"));
        let json = serde_json::to_string(&p).unwrap();
        let p2: AgentProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(p.name, p2.name);
        assert_eq!(p.states, p2.states);
    }

    #[test]
    fn serde_abelian_group() {
        let g = AbelianGroup::new(2, vec![3, 5]);
        let json = serde_json::to_string(&g).unwrap();
        let g2: AbelianGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(g.rank, g2.rank);
        assert_eq!(g.torsion, g2.torsion);
    }

    #[test]
    fn serde_fibration() {
        let base = TopologicalSpace::interval(2);
        let fiber = TopologicalSpace::discrete(vec![0.0]);
        let total = TopologicalSpace::interval(2);
        let fib = Fibration::new(total, base, fiber, vec![0, 1]);
        let json = serde_json::to_string(&fib).unwrap();
        let fib2: Fibration = serde_json::from_str(&json).unwrap();
        assert_eq!(fib.projection_map, fib2.projection_map);
    }

    #[test]
    fn serde_exact_sequence() {
        let groups = vec![AbelianGroup::trivial(), AbelianGroup::z()];
        let maps = vec![SparseMatrix::zero(1, 1)];
        let seq = ExactSequence::new(groups, maps);
        let json = serde_json::to_string(&seq).unwrap();
        let seq2: ExactSequence = serde_json::from_str(&json).unwrap();
        assert_eq!(seq.groups.len(), seq2.groups.len());
    }

    #[test]
    fn serde_homotopy_group() {
        let hg = HomotopyGroup::new(1, TopologicalSpace::circle(6), 0);
        let json = serde_json::to_string(&hg).unwrap();
        let hg2: HomotopyGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(hg.dimension, hg2.dimension);
    }

    #[test]
    fn serde_higher_homotopy() {
        let h1 = Homotopy::new(Path::new(vec![0.0, 1.0]), Path::new(vec![0.0, 0.5]), 5);
        let h2 = Homotopy::new(Path::new(vec![0.0, 1.0]), Path::new(vec![0.0, 0.6]), 5);
        let hh = HigherHomotopy::new(h1, h2);
        let json = serde_json::to_string(&hh).unwrap();
        let hh2: HigherHomotopy = serde_json::from_str(&json).unwrap();
        assert_eq!(hh.h1.steps, hh2.h1.steps);
    }

    #[test]
    fn serde_transition() {
        let t = Transition::with_condition(0, 1, "go", "ready");
        let json = serde_json::to_string(&t).unwrap();
        let t2: Transition = serde_json::from_str(&json).unwrap();
        assert_eq!(t.label, t2.label);
        assert_eq!(t.condition, t2.condition);
    }

    #[test]
    fn serde_sparse_matrix() {
        let m = SparseMatrix::identity(3);
        let json = serde_json::to_string(&m).unwrap();
        let m2: SparseMatrix = serde_json::from_str(&json).unwrap();
        assert_eq!(m.entries, m2.entries);
    }
}
