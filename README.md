# lau-categorical-homotopy

> Homotopy type theory meets agent systems — paths, homotopies, fundamental groupoids, and higher inductive types as agent communication protocols

## What This Does

Homotopy type theory meets agent systems — paths, homotopies, fundamental groupoids, and higher inductive types as agent communication protocols. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-categorical-homotopy
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_categorical_homotopy::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct TopologicalSpace 
    pub fn new(points: Vec<f64>, adjacency: Vec<Vec<usize>>) -> Self 
    pub fn discrete(points: Vec<f64>) -> Self 
    pub fn circle(n: usize) -> Self 
    pub fn interval(n: usize) -> Self 
    pub fn connected_components(&self) -> Vec<Vec<usize>> 
    pub fn contractible(&self) -> bool 
    pub fn euler_characteristic(&self) -> i32 
    pub fn dimension(&self) -> usize 
    pub fn nearest(&self, x: f64) -> usize 
pub struct Path 
    pub fn new(points: Vec<f64>) -> Self 
    pub fn constant(x: f64, n: usize) -> Self 
    pub fn start(&self) -> f64 
    pub fn end(&self) -> f64 
    pub fn evaluate(&self, t: f64) -> f64 
    pub fn concat(&self, other: &Path) -> Option<Path> 
    pub fn reverse(&self) -> Path 
    pub fn reparametrize(&self, n: usize) -> Path 
    pub fn length(&self) -> f64 
pub struct Homotopy 
    pub fn new(from: Path, to: Path, steps: usize) -> Self 
    pub fn intermediate(&self, t: f64) -> Path 
    pub fn is_valid(&self) -> bool 
    pub fn endpoint_preserving(&self) -> bool 
    pub fn relative(&self, subspace: &[f64]) -> bool 
pub struct FundamentalGroupoid 
    pub fn new(space: TopologicalSpace, base_points: Vec<usize>) -> Self 
    pub fn homotopy_class(&self, path: &Path) -> usize 
    pub fn compose(&self, class_a: usize, class_b: usize) -> Option<usize> 
    pub fn inverse(&self, class: usize) -> usize 
    pub fn is_invertible(&self, _class: usize) -> bool 
    pub fn fundamental_group(&self, base_point: usize) -> Vec<usize> 
    pub fn is_simply_connected(&self) -> bool 
pub struct HigherHomotopy 
    pub fn new(h1: Homotopy, h2: Homotopy) -> Self 
    pub fn is_valid(&self) -> bool 
    pub fn intermediate(&self, s: f64, t: f64) -> f64 
pub struct HomotopyGroup 
    pub fn new(dimension: usize, space: TopologicalSpace, base_point: usize) -> Self 
    pub fn group_operation(&self, a: &[f64], b: &[f64]) -> Vec<f64> 
    pub fn identity(&self) -> Vec<f64> 
    pub fn inverse(&self, element: &[f64]) -> Vec<f64> 
    pub fn order(&self) -> usize 
pub struct AbelianGroup 
    pub fn new(rank: usize, torsion: Vec<usize>) -> Self 
    pub fn trivial() -> Self 
    pub fn z() -> Self 
    pub fn zn(n: usize) -> Self 
    pub fn order(&self) -> Option<usize> 
    pub fn is_trivial(&self) -> bool 
    pub fn direct_sum(&self, other: &AbelianGroup) -> AbelianGroup 
pub struct SparseMatrix 
    pub fn zero(rows: usize, cols: usize) -> Self 
    pub fn identity(n: usize) -> Self 
    pub fn apply(&self, v: &[i64]) -> Vec<i64> 
    pub fn image(&self) -> Vec<Vec<i64>> 
    pub fn kernel(&self) -> Vec<Vec<i64>> 
pub struct ExactSequence 
    pub fn new(groups: Vec<AbelianGroup>, maps: Vec<SparseMatrix>) -> Self 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**81 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
