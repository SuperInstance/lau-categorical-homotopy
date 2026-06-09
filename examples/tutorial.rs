//! # Categorical Homotopy — Tutorial
//!
//! Progressive lessons covering homotopy type theory, paths, homotopies,
//! fundamental groupoids, higher inductive types, and agent protocols.
//!
//! Run with: `cargo run --example tutorial`

use lau_categorical_homotopy::{
    AbelianGroup, AgentProtocol, ExactSequence, Fibration, FundamentalGroupoid,
    Homotopy, HomotopyGroup, Path, SparseMatrix, SyntheticType, TopologicalSpace, Transition,
};

// ── Lesson 1: Topological Spaces ─────────────────────────────────────────────
//
// TopologicalSpace provides a finite simplicial complex approximation of a space.
// Factory methods build common shapes: circles, intervals, discrete spaces.

fn lesson_1_topological_spaces() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 1: Topological Spaces");
    println!("═══════════════════════════════════════════\n");

    // Circle S¹: 6 equally-spaced points on [0, 1) with wrap-around adjacency
    let circle = TopologicalSpace::circle(6);
    println!("Circle S¹ (6 points):");
    println!("  points = {:?}", circle.points);
    println!("  adjacency = {:?}", circle.adjacency);
    println!("  connected components = {:?}", circle.connected_components());
    println!("  contractible? {}", circle.contractible());
    println!("  Euler characteristic χ = {}", circle.euler_characteristic());
    println!("  dimension = {}", circle.dimension());

    // Interval [0, 1]: a tree (contractible)
    let interval = TopologicalSpace::interval(4);
    println!("\nInterval [0,1] (4 points):");
    println!("  points = {:?}", interval.points);
    println!("  contractible? {}", interval.contractible());
    println!("  Euler characteristic χ = {}", interval.euler_characteristic());

    // Discrete space: isolated points
    let discrete = TopologicalSpace::discrete(vec![0.0, 1.0, 2.0]);
    println!("\nDiscrete space {{0, 1, 2}}:");
    println!("  connected components = {:?}", discrete.connected_components());
    println!("  contractible? {}", discrete.contractible());
    println!("  dimension = {}", discrete.dimension());
    println!();
}

// ── Lesson 2: Paths and Concatenation ────────────────────────────────────────
//
// A Path is a continuous map [0,1] → X, stored as sampled polyline points.
// Paths can be evaluated at any parameter, concatenated, and reversed.

fn lesson_2_paths() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 2: Paths and Concatenation");
    println!("═══════════════════════════════════════════\n");

    let p = Path::new(vec![0.0, 0.5, 1.0]);
    println!("Path p = [0.0, 0.5, 1.0]:");
    println!("  start = {:.1}, end = {:.1}", p.start(), p.end());
    println!("  length = {:.4}", p.length());
    println!("  evaluate(0.0) = {:.4}", p.evaluate(0.0));
    println!("  evaluate(0.5) = {:.4}", p.evaluate(0.5));
    println!("  evaluate(1.0) = {:.4}", p.evaluate(1.0));

    // Concatenation: p · q (requires p.end == q.start)
    let q = Path::new(vec![1.0, 1.5, 2.0]);
    let pq = p.concat(&q).unwrap();
    println!("\nPath q = [1.0, 1.5, 2.0]");
    println!("Concatenated p·q = {:?} (length = {:.4})", pq.points, pq.length());

    // Failed concatenation (endpoints don't match)
    let r = Path::new(vec![0.5, 1.5]);
    println!("\nCan we concat p · [0.5, 1.5]? {:?}", p.concat(&r));

    // Reverse
    let rev = p.reverse();
    println!("Reverse of p = {:?}", rev.points);

    // Round trip: p · reverse(p) is a loop
    let loop_path = p.concat(&rev).unwrap();
    println!("p · reverse(p) = {:?} (a loop!)", loop_path.points);
    println!("  start = {:.1}, end = {:.1}", loop_path.start(), loop_path.end());

    // Reparametrize to different resolution
    let reparam = p.reparametrize(5);
    println!("\nReparametrize p to 5 points: {:?}", reparam.points);

    // Constant path
    let c = Path::constant(3.0, 4);
    println!("Constant path at 3.0: {:?} (length = {:.4})", c.points, c.length());
    println!();
}

// ── Lesson 3: Homotopies — Continuous Deformations ──────────────────────────
//
// A Homotopy is a continuous deformation between two paths that share endpoints.
// H(t) interpolates between from_path and to_path, with H(0) = from, H(1) = to.

fn lesson_3_homotopies() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 3: Homotopies — Continuous Deformations");
    println!("═══════════════════════════════════════════\n");

    let p1 = Path::new(vec![0.0, 0.3, 1.0]);
    let p2 = Path::new(vec![0.0, 0.9, 1.0]);
    let h = Homotopy::new(p1.clone(), p2.clone(), 10);

    println!("Homotopy between two paths from 0 to 1:");
    println!("  from = {:?} (low route)", p1.points);
    println!("  to   = {:?} (high route)", p2.points);
    println!("  is valid? {} (same endpoints)", h.is_valid());
    println!("  endpoint preserving? {}", h.endpoint_preserving());

    // Intermediate paths at different t values
    println!("\nIntermediate paths during deformation:");
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let mid = h.intermediate(t);
        let sample: Vec<String> = mid.points.iter().map(|x| format!("{:.2}", x)).collect();
        println!("  H(t={:.2}) = [{}]", t, sample.join(", "));
    }

    // Invalid homotopy: different endpoints
    let p3 = Path::new(vec![0.0, 0.5, 2.0]); // different endpoint
    let h_bad = Homotopy::new(p1.clone(), p3, 10);
    println!("\nHomotopy with mismatched endpoint: valid? {}", h_bad.is_valid());

    // Relative homotopy
    println!("\nRelative homotopy (fixed subspace {{0.0, 1.0}}): {}", h.relative(&[0.0, 1.0]));
    println!();
}

// ── Lesson 4: Fundamental Groupoids ─────────────────────────────────────────
//
// The FundamentalGroupoid captures all paths up to homotopy equivalence.
// For contractible spaces, all paths between the same two points are homotopic.
// For S¹, different winding numbers give different homotopy classes.

fn lesson_4_fundamental_groupoids() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 4: Fundamental Groupoids");
    println!("═══════════════════════════════════════════\n");

    // On a contractible space (interval), all paths are homotopic
    let interval = TopologicalSpace::interval(10);
    let fg_interval = FundamentalGroupoid::new(interval.clone(), vec![0, 5, 9]);
    println!("Fundamental groupoid of the interval [0, 1]:");
    println!("  simply connected? {}", fg_interval.is_simply_connected());

    let p1 = Path::new(vec![0.0, 0.5, 1.0]);
    let p2 = Path::new(vec![0.0, 0.8, 1.0]);
    let c1 = fg_interval.homotopy_class(&p1);
    let c2 = fg_interval.homotopy_class(&p2);
    println!("  class({:?}) = {}", p1.points, c1);
    println!("  class({:?}) = {}", p2.points, c2);
    println!("  Same class? {} (all paths homotopic on contractible space)", c1 == c2);

    // On S¹, winding number matters
    let circle = TopologicalSpace::circle(8);
    let fg_circle = FundamentalGroupoid::new(circle.clone(), vec![0]);
    println!("\nFundamental groupoid of S¹:");
    println!("  simply connected? {}", fg_circle.is_simply_connected());

    // Groupoid composition
    let space = TopologicalSpace::interval(10);
    let fg = FundamentalGroupoid::new(space.clone(), vec![0, 3, 6, 9]);
    let c01 = fg.homotopy_class(&Path::new(vec![space.points[0], space.points[3]]));
    let c12 = fg.homotopy_class(&Path::new(vec![space.points[3], space.points[6]]));
    let composed = fg.compose(c01, c12);
    println!("\nGroupoid composition on interval:");
    println!("  class(0→3) ∘ class(3→6) = {:?}", composed);

    // Inverse
    let inv = fg.inverse(c01);
    println!("  inverse of class(0→3) = {}", inv);
    println!("  all morphisms invertible? {}", fg.is_invertible(c01));

    // Fundamental group π₁
    let pi1 = fg.fundamental_group(0);
    println!("  π₁ at base 0: {:?}", pi1);
    println!();
}

// ── Lesson 5: Abelian Groups and Exact Sequences ────────────────────────────
//
// Homotopy groups are abelian groups (for n ≥ 2). We model them as
// Z^rank ⊕ Z/t₁ ⊕ Z/t₂ ⊕ ... and build exact sequences.

fn lesson_5_abelian_groups_and_exact_sequences() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 5: Abelian Groups & Exact Sequences");
    println!("═══════════════════════════════════════════\n");

    // Build some abelian groups
    let trivial = AbelianGroup::trivial();
    let z = AbelianGroup::z();
    let z5 = AbelianGroup::zn(5);
    let z2z3 = AbelianGroup::zn(2).direct_sum(&AbelianGroup::zn(3));

    println!("Abelian groups:");
    println!("  trivial: rank={}, torsion={:?}, order={:?}", trivial.rank, trivial.torsion, trivial.order());
    println!("  Z: rank={}, torsion={:?}, order={:?}", z.rank, z.torsion, z.order());
    println!("  Z/5: rank={}, torsion={:?}, order={:?}", z5.rank, z5.torsion, z5.order());
    println!("  Z/2 ⊕ Z/3: rank={}, torsion={:?}, order={:?}", z2z3.rank, z2z3.torsion, z2z3.order());

    // Build a short exact sequence: 0 → Z → Z⊕Z → Z → 0
    let groups = vec![
        AbelianGroup::trivial(),
        AbelianGroup::z(),
        AbelianGroup::new(2, vec![]),
        AbelianGroup::z(),
        AbelianGroup::trivial(),
    ];
    // Inclusion: Z → Z⊕Z maps generator to first component
    let incl = SparseMatrix {
        rows: 2, cols: 1,
        entries: vec![(0, 0, 1), (1, 0, 0)],
    };
    // Projection: Z⊕Z → Z projects onto second component
    let proj = SparseMatrix {
        rows: 1, cols: 2,
        entries: vec![(0, 1, 1)],
    };
    // Trivial maps at ends
    let zero_01 = SparseMatrix::zero(1, 1);
    let zero_34 = SparseMatrix::zero(1, 1);

    let seq = ExactSequence::new(groups, vec![zero_01, incl, proj, zero_34]);
    println!("\nShort exact sequence: 0 → Z → Z⊕Z → Z → 0");
    println!("  is exact? {}", seq.is_exact());
    println!("  homology dimensions: {:?}", seq.compute_homology());
    println!();
}

// ── Lesson 6: Homotopy Groups and Fibrations ────────────────────────────────
//
// The n-th homotopy group πₙ(X, x₀) classifies n-dimensional "loops" in X.
// Fibrations (fiber bundles) give rise to long exact sequences of homotopy groups.

fn lesson_6_homotopy_groups_and_fibrations() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 6: Homotopy Groups & Fibrations");
    println!("═══════════════════════════════════════════\n");

    // π₀: connected components
    let discrete = TopologicalSpace::discrete(vec![0.0, 1.0, 2.0, 3.0]);
    let pi0 = HomotopyGroup::new(0, discrete, 0);
    println!("π₀(discrete 4-point space):");
    println!("  order = {} (4 connected components)", pi0.order());
    println!("  identity = {:?}", pi0.identity());

    // π₁: fundamental group of S¹
    let circle = TopologicalSpace::circle(8);
    let pi1 = HomotopyGroup::new(1, circle.clone(), 0);
    println!("\nπ₁(S¹, base=0):");
    println!("  order = {}", pi1.order());
    println!("  identity = {:?}", pi1.identity());

    // Group operation
    let a = vec![1.0];
    let b = vec![2.0];
    println!("  [1] + [2] = {:?}", pi1.group_operation(&a, &b));
    println!("  -[1] = {:?}", pi1.inverse(&a));

    // Higher homotopy groups (trivial for our discrete spaces)
    let pi2 = HomotopyGroup::new(2, circle, 0);
    println!("\nπ₂(S¹) (higher homotopy group):");
    println!("  elements = {:?}", pi2.elements);
    println!("  order = {}", pi2.order());

    // Fibration: trivial fiber bundle
    let base = TopologicalSpace::interval(3);
    let fiber = TopologicalSpace::discrete(vec![0.0, 1.0]);
    let total_points = vec![0.0, 1.0, 0.5, 1.5, 1.0, 2.0];
    let total = TopologicalSpace::discrete(total_points);
    let projection = vec![0, 0, 1, 1, 2, 2]; // pairs map to same base
    let fib = Fibration::new(total, base, fiber, projection);

    println!("\nFiber bundle: total → base (fiber = 2-point discrete):");
    println!("  is fiber bundle? {}", fib.is_fiber_bundle());
    println!("  fiber over base point 0: {:?}", fib.fiber_over(0));
    println!("  fiber over base point 1: {:?}", fib.fiber_over(1));
    println!("  Serre fibration? {}", fib.serre_fibration());

    let seq = fib.exact_sequence();
    println!("  Long exact sequence has {} groups", seq.groups.len());
    println!();
}

// ── Lesson 7: Synthetic Types (Homotopy Type Theory) ────────────────────────
//
// In HoTT, types have h-levels: Contractible (0), Proposition (1), Set (2),
// Groupoid (3), HigherGroupoid (n≥4). The identity type lowers h-level by 1.

fn lesson_7_synthetic_types() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 7: Synthetic Types (HoTT)");
    println!("═══════════════════════════════════════════\n");

    let contractible = SyntheticType::contractible();
    let proposition = SyntheticType::proposition();
    let set = SyntheticType::set();
    let groupoid = SyntheticType::groupoid();
    let higher = SyntheticType::higher(5);

    println!("Type hierarchy (h-levels):");
    for (name, ty) in [("Contractible", &contractible), ("Proposition", &proposition),
                        ("Set", &set), ("Groupoid", &groupoid), ("Higher(5)", &higher)] {
        println!("  {}: h-level={}, is_prop={}, is_set={}, is_groupoid={}",
            name, ty.h_level(), ty.is_prop(), ty.is_set(), ty.is_groupoid());
    }

    // Identity types: each type's equality type drops one h-level
    println!("\nIdentity type computation:");
    println!("  Id(Contractible) = h-level {} (Contractible)", contractible.identity_type().h_level());
    println!("  Id(Proposition)  = h-level {} (Contractible)", proposition.identity_type().h_level());
    println!("  Id(Set)          = h-level {} (Proposition)", set.identity_type().h_level());
    println!("  Id(Groupoid)     = h-level {} (Set)", groupoid.identity_type().h_level());
    println!("  Id(Higher(5))    = h-level {} (Higher(4))", higher.identity_type().h_level());

    // Truncation: lower a type to a specific h-level
    println!("\nTruncation of Higher(5):");
    for level in 0..=5 {
        let trunc = higher.truncation(level);
        println!("  ||Higher(5)||_{} = h-level {}", level, trunc.h_level());
    }
    println!();
}

// ── Lesson 8: Agent Protocols as Higher Inductive Types ──────────────────────
//
// An AgentProtocol models communication as a higher inductive type.
// States are points; transitions are paths. Protocol correctness means
// all execution paths are homotopic (unique up to homotopy).

fn lesson_8_agent_protocols() {
    println!("═══════════════════════════════════════════");
    println!("  Lesson 8: Agent Protocols");
    println!("═══════════════════════════════════════════\n");

    // Simple two-phase commit protocol
    let mut proto = AgentProtocol::new("two-phase-commit", SyntheticType::set());
    let idle = proto.add_state("idle");
    let prepared = proto.add_state("prepared");
    let committed = proto.add_state("committed");
    let aborted = proto.add_state("aborted");

    proto.add_transition(Transition::new(idle, prepared, "prepare"));
    proto.add_transition(Transition::new(prepared, committed, "commit"));
    proto.add_transition(Transition::new(prepared, aborted, "abort"));

    println!("Protocol: two-phase-commit");
    println!("  States: {:?}", proto.states);
    println!("  Transitions:");
    for t in &proto.transitions {
        println!("    {} --[{}]--> {}",
            proto.states[t.from_state], t.label, proto.states[t.to_state]);
    }
    println!("  Is correct? {}", proto.is_correct());
    println!("  Deadlock free? {}", proto.deadlock_free());

    // Correct protocol: all states reachable
    let mut good = AgentProtocol::new("pipeline", SyntheticType::contractible());
    let s0 = good.add_state("init");
    let s1 = good.add_state("process");
    let s2 = good.add_state("done");
    good.add_transition(Transition::new(s0, s1, "start"));
    good.add_transition(Transition::new(s1, s2, "finish"));
    println!("\nLinear pipeline protocol:");
    println!("  Is correct? {}", good.is_correct());
    println!("  Deadlock free? {}", good.deadlock_free());
    println!("  (Contractible path type => guaranteed deadlock-free)");

    // Protocol with verification paths
    let mut verified = AgentProtocol::new("verified-handshake", SyntheticType::set());
    let v0 = verified.add_state("closed");
    let v1 = verified.add_state("syn_sent");
    let v2 = verified.add_state("established");
    verified.add_transition(Transition::with_condition(v0, v1, "SYN", "init_ready"));
    verified.add_transition(Transition::with_condition(v1, v2, "SYN-ACK", "ack_received"));
    verified.verification_paths.push(Path::new(vec![0.0, 1.0, 2.0]));

    println!("\nVerified handshake protocol:");
    println!("  States: {:?}", verified.states);
    println!("  Verification paths: {}", verified.verification_paths.len());
    println!("  Correct? {} Deadlock-free? {}", verified.is_correct(), verified.deadlock_free());
    println!();
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  Categorical Homotopy — Interactive Tutorial          ║");
    println!("║  From paths to homotopy type theory & agent protocols ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    lesson_1_topological_spaces();
    lesson_2_paths();
    lesson_3_homotopies();
    lesson_4_fundamental_groupoids();
    lesson_5_abelian_groups_and_exact_sequences();
    lesson_6_homotopy_groups_and_fibrations();
    lesson_7_synthetic_types();
    lesson_8_agent_protocols();

    println!("═════════════════════════════════════════════════");
    println!("  ✓ Tutorial complete — all 8 lessons done!");
    println!("═════════════════════════════════════════════════");
}
