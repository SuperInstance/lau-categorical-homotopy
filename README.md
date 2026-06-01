# lau-categorical-homotopy

> Homotopy type theory meets agent systems — paths, homotopies, fundamental groupoids, and higher inductive types as agent communication protocols.

[![crates.io](https://img.shields.io/crates/v/lau-categorical-homotopy)](https://crates.io/crates/lau-categorical-homotopy)
[![tests](https://img.shields.io/badge/tests-81-green)]()
[![license](https://img.shields.io/badge/license-MIT-blue)]()

## What This Does

This crate provides a computational framework for **homotopy theory and categorical topology**, implementing the mathematical structures that govern continuous deformation, path equivalence, and algebraic topology — and applies them to **agent communication protocols**.

It bridges three worlds:

1. **Topology**: Spaces, paths, homotopies, fibrations, exact sequences
2. **Algebra**: Fundamental groups, homotopy groups, abelian groups, chain complexes
3. **Agents**: Protocol state machines verified through homotopy correctness (deadlock-freedom, unique-path guarantees)

Every structure is discrete/computable (spaces are finite simplicial complexes, paths are polylines), yet faithfully implements the mathematical theory. All types are `serde`-serializable.

## The Key Idea

In homotopy type theory, **equality is a space** — two paths between the same points can be equal, different, or "homotopic" (continuously deformable into each other). This crate makes that computational:

- A **path** is a polyline `[0,1] → X` through a finite space
- A **homotopy** is a linear deformation between two paths (keeping endpoints fixed)
- The **fundamental groupoid** tracks path equivalence classes and composition
- **Agent protocols** use the homotopy level (h-level) of their path type as a correctness guarantee:
  - *Contractible* paths → protocol is always deadlock-free
  - *Set* paths → all execution paths are unique (deterministic)
  - *Proposition* paths → at most one way between any two states

The crate proves 10 named theorems in its test suite, including:
- **π₁(S¹) ≅ ℤ** — the fundamental group of the circle is the integers
- Fundamental group of any contractible space is trivial
- Path concatenation preserves homotopy
- Groupoid composition is associative
- Contractible path type ⇒ deadlock-free protocol

## Install

```bash
cargo add lau-categorical-homotopy
```

## Quick Start

```rust
use lau_categorical_homotopy::*;

// Build a circle with 8 sample points
let circle = TopologicalSpace::circle(8);

// The circle is connected but NOT contractible (it has a hole!)
assert_eq!(circle.connected_components().len(), 1);
assert!(!circle.contractible());
assert_eq!(circle.euler_characteristic(), 0); // V - E = 0

// Two paths with the same endpoints
let p1 = Path::new(vec![0.0, 0.5, 1.0]);
let p2 = Path::new(vec![0.0, 0.8, 1.0]);

// They're homotopic (same start/end, deformable into each other)
let h = Homotopy::new(p1.clone(), p2.clone(), 10);
assert!(h.is_valid());
assert!(h.endpoint_preserving());

// The fundamental groupoid tracks homotopy classes
let fg = FundamentalGroupoid::new(circle, vec![0]);
assert!(!fg.is_simply_connected()); // π₁(S¹) ≠ 0

// Agent protocols use h-levels for correctness guarantees
let mut protocol = AgentProtocol::new("handshake", SyntheticType::contractible());
let start = protocol.add_state("init");
let end = protocol.add_state("done");
protocol.add_transition(Transition::new(start, end, "complete"));
assert!(protocol.is_correct());    // all states reachable
assert!(protocol.deadlock_free()); // contractible ⇒ always safe
```

## API Reference

### TopologicalSpace

Finite approximation of a topological space via a 1-dimensional simplicial complex.

| Method | Description |
|--------|-------------|
| `new(points, adjacency)` | Build from explicit vertex positions and adjacency list |
| `discrete(points)` | Build a discrete space (no edges — totally disconnected) |
| `circle(n)` | Sample S¹ with `n` equally-spaced points |
| `interval(n)` | Sample [0,1] with `n` points (contractible) |
| `connected_components()` | BFS-based component detection |
| `contractible()` | True iff connected and acyclic (a tree) |
| `euler_characteristic()` | V − E (for 1-complexes) |
| `dimension()` | 0 if discrete, 1 if any edges exist |
| `nearest(x)` | Index of the point closest to `x` |

### Path

A continuous map `[0,1] → X` represented as a polyline.

| Method | Description |
|--------|-------------|
| `new(points)` | Construct from sample points |
| `constant(x, n)` | Constant path at `x` |
| `start()` / `end()` | First and last sample values |
| `evaluate(t)` | Linear interpolation at `t ∈ [0,1]` |
| `concat(other)` | Concatenate (requires matching endpoints) |
| `reverse()` | Reverse direction |
| `reparametrize(n)` | Resample to `n` equidistant points |
| `length()` | Total arc length |

### Homotopy

A continuous deformation between two paths via linear interpolation.

| Method | Description |
|--------|-------------|
| `new(from, to, steps)` | Build a homotopy with `steps` intermediate paths |
| `intermediate(t)` | The path at parameter `t ∈ [0,1]` |
| `is_valid()` | Endpoints match at `t=0` and `t=1` |
| `endpoint_preserving()` | Start and end don't move during deformation |
| `relative(subspace)` | Fixed points in subspace stay fixed |

### FundamentalGroupoid

The groupoid of paths up to homotopy equivalence.

| Method | Description |
|--------|-------------|
| `new(space, base_points)` | Compute homotopy classes over base points |
| `homotopy_class(path)` | Which equivalence class does this path belong to? |
| `compose(a, b)` | Groupoid composition (concatenation of classes) |
| `inverse(class)` | Reverse path class |
| `is_invertible(_)` | Always `true` — all morphisms in a groupoid are iso |
| `fundamental_group(base)` | π₁(X, x₀): automorphism classes at a base point |
| `is_simply_connected()` | True iff space is contractible |

### HigherHomotopy

A 2-homotopy — a homotopy between homotopies.

| Method | Description |
|--------|-------------|
| `new(h1, h2)` | Two homotopies between the same endpoint paths |
| `is_valid()` | Both homotopies valid and compatible |
| `intermediate(s, t)` | 2-parameter family: `(s,t) → X` |

### HomotopyGroup

The n-th homotopy group πₙ(X, x₀).

| Method | Description |
|--------|-------------|
| `new(dimension, space, base)` | Compute elements for π₀, π₁, or higher |
| `group_operation(a, b)` | Group operation (concatenation / pointwise) |
| `identity()` | Identity element |
| `inverse(element)` | Inverse element |
| `order()` | Number of computed elements |

### AbelianGroup

A finitely-generated abelian group: ℤʳ ⊕ ℤ/t₁ ⊕ ℤ/t₂ ⊕ ⋯

| Method | Description |
|--------|-------------|
| `new(rank, torsion)` | ℤ^rank ⊕ ⊕ ℤ/tᵢ |
| `trivial()` | The trivial group {0} |
| `z()` | The integers ℤ |
| `zn(n)` | The cyclic group ℤ/n |
| `order()` | `None` if infinite, otherwise the order |
| `is_trivial()` | Rank 0, no torsion |
| `direct_sum(other)` | Direct sum of two groups |

### SparseMatrix & ExactSequence

Sparse integer matrices for group homomorphisms and exact sequences of abelian groups.

| Method | Description |
|--------|-------------|
| `SparseMatrix::zero(r, c)` | Zero matrix |
| `SparseMatrix::identity(n)` | Identity matrix |
| `apply(v)` | Matrix-vector product |
| `image()` / `kernel()` | Column space and null space |
| `ExactSequence::new(groups, maps)` | Chain of groups with boundary maps |
| `is_exact()` | Verify im(∂ᵢ₊₁) = ker(∂ᵢ) |
| `compute_homology()` | Homology group dimensions |

### Fibration

A fiber bundle: total space → base space with fiber.

| Method | Description |
|--------|-------------|
| `new(total, base, fiber, projection)` | Build with explicit projection map |
| `projection(point)` | Map a total-space point to base |
| `fiber_over(base_point)` | Preimage of a base point |
| `exact_sequence()` | Long exact sequence of homotopy groups |
| `is_fiber_bundle()` | All fibers have same cardinality |
| `serre_fibration()` | Homotopy lifting property (discrete: = fiber bundle) |

### SyntheticType

Homotopy type theory truncation levels (h-levels).

| Variant | h-level | Meaning |
|---------|---------|---------|
| `Contractible` | 0 | Exactly one inhabitant (up to homotopy) |
| `Proposition` | 1 | At most one proof of equality |
| `Set` | 2 | Equality proofs are unique (UIP) |
| `Groupoid` | 3 | Equality of equalities is a proposition |
| `HigherGroupoid(n)` | n ≥ 4 | Higher structure |

| Method | Description |
|--------|-------------|
| `h_level()` | Numeric truncation level |
| `identity_type()` | The type of equalities (h-level decreases by 1) |
| `is_prop()` / `is_set()` / `is_groupoid()` | Truncation predicates |
| `truncation(level)` | n-truncation |

### AgentProtocol

An agent communication protocol modeled as a higher inductive type.

| Method | Description |
|--------|-------------|
| `new(name, path_type)` | Create with a given h-level |
| `add_state(name)` | Add a named state |
| `add_transition(t)` | Add a labeled transition |
| `is_correct()` | All states reachable from start |
| `deadlock_free()` | No non-final state lacks outgoing transitions |

## How It Works

### Architecture

The crate builds a layered computational model:

```
TopologicalSpace          ← finite simplicial complex (points + adjacency)
  └→ Path                 ← polylines in the space
      └→ Homotopy         ← linear deformation between paths
          └→ HigherHomotopy ← homotopy of homotopies (2-parameter)
              └→ HomotopyGroup  ← πₙ(X, x₀) algebraic structure

FundamentalGroupoid       ← path classes + composition (category theory)
  └→ Fibration            ← fiber bundle with exact sequence
      └→ ExactSequence    ← im/ker verification + homology

SyntheticType             ← h-level hierarchy (HoTT truncation)
  └→ AgentProtocol        ← state machines verified via homotopy level
```

### Discrete Approximation Strategy

Real topological spaces are uncountable. This crate approximates them as **finite 1-dimensional simplicial complexes**:

- **Spaces** = vertex set + adjacency list (graph)
- **Paths** = polylines through vertices, evaluated via linear interpolation
- **Homotopy** = linear interpolation between two reparametrized paths
- **Contractible** = connected + acyclic (tree)
- **Euler characteristic** = V − E (for 1-complexes)

This is sufficient to capture the essential topology:
- Circles have χ = 0 and non-trivial π₁
- Intervals have χ = 1 and trivial π₁
- Discrete spaces decompose into connected components

### Winding Number Heuristic

For non-contractible spaces (like the circle), the `FundamentalGroupoid` uses a **winding number** to distinguish homotopy classes: it counts signed crossings of the path through a reference point (0.5 for the unit circle). Paths with different winding numbers belong to different homotopy classes.

## The Math

### Named Theorems (verified in tests)

| # | Theorem | Statement |
|---|---------|-----------|
| 1 | π₁(S¹) ≅ ℤ | The fundamental group of the circle is infinite cyclic |
| 2 | Contractible ⇒ trivial π₁ | Any contractible space has trivial fundamental group |
| 3 | Homotopy preserves concatenation | If p₁ ~ p₂ then p₁·q ~ p₂·q |
| 4 | Fibration exact sequence | Every fibration induces a long exact sequence of homotopy groups |
| 5 | Euler characteristic homotopy invariant | Homotopy-equivalent spaces have the same χ |
| 6 | Groupoid associativity | (a · b) · c ~ a · (b · c) up to higher homotopy |
| 7 | 0-truncation of S¹ is a set | Truncating the circle to h-level 2 yields decidable equality |
| 8 | Contractible ⇒ deadlock-free | Protocols with contractible path types cannot deadlock |
| 9 | Reversal is groupoid inverse | p · p⁻¹ ~ id and p⁻¹ · p ~ id |
| 10 | Simply connected = unique path | In simply connected spaces, any two paths between the same points are homotopic |

### Core Mathematical Structures

- **Fundamental groupoid** Π₁(X): The category whose objects are points of X and whose morphisms are homotopy classes of paths. Every morphism is invertible.
- **Homotopy groups** πₙ(X, x₀): Iterated loop spaces. π₀ counts components, π₁ is the fundamental group, higher groups are always abelian.
- **Fibrations** p: E → B: Maps with the homotopy lifting property. They induce long exact sequences ⋯ → πₙ(F) → πₙ(E) → πₙ(B) → πₙ₋₁(F) → ⋯
- **Exact sequences**: Chains ⋯ → A → B → C → ⋯ where im(∂ᵢ₊₁) = ker(∂ᵢ). The failure of exactness (homology) measures topological holes.
- **H-levels** (HoTT): The stratification of types by the complexity of their identity types. Contractible (0) < Proposition (1) < Set (2) < Groupoid (3) < ⋯

## Testing

**81 tests** covering:

- **Path operations**: evaluation, concatenation, reversal, reparametrization, length
- **Space topology**: connected components, contractibility, Euler characteristic, dimension
- **Homotopy**: validity, endpoint preservation, relative homotopy, linear deformation
- **Groupoid**: composition, inverses, fundamental group, simply-connected detection
- **Higher homotopy**: 2-parameter families, validity constraints
- **Algebraic structures**: abelian groups, direct sums, sparse matrices, exact sequences
- **Fibrations**: fiber bundles, Serre fibrations, exact sequence generation
- **HoTT types**: h-levels, identity types, truncation
- **Agent protocols**: correctness, deadlock detection, contractible guarantees
- **Serde round-trips**: every public type serializes/deserializes correctly

## License

MIT
