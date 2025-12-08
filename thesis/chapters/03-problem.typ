= Problem Formalization <ch:problem>

In this chapter we formally define the computational problems that arise from the Top Trading Cycles mechanism in the context of GP allocation. We show how the problem maps to cycle cover problems on directed graphs.

== Graph Construction from TTC Instance

Consider a TTC instance with a set of patients $P$ and a set of doctors $D$. Each patient $p in P$ has:
- A _current doctor_ $d_p in D union {bot}$, where $bot$ denotes that the patient is currently unassigned
- A _preferred doctor_ $d_p^* in D$

We say a patient _wants to switch_ if $d_p != d_p^*$.

#block(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Definition 3.1 (TTC Preference Graph).* Given a TTC instance $(P, D, d, d^*)$, the _preference graph_ is the directed graph $G = (V, E)$ where:
  - $V = {p in P : d_p != d_p^*}$ #h(1em) (patients wanting to switch)
  - $E = {(p, q) : d_p^* = d_q}$ #h(1em) (edge from $p$ to $q$ if $p$ wants $q$'s current doctor)
]

An edge $(p, q)$ represents that patient $p$ would benefit from acquiring patient $q$'s doctor assignment. A directed cycle $C = (p_1 -> p_2 -> dots.c -> p_ell -> p_1)$ in this graph represents a valid exchange: patient $p_i$ receives $p_(i+1)$'s doctor (with indices taken modulo $ell$). After executing such a cycle, all participating patients receive their preferred doctor.

== Cycle Cover Problems

We first establish the standard terminology for cycle covers in directed graphs.

#block(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Definition 3.2 (Cycle Cover).* A _cycle cover_ of a directed graph $G = (V, E)$ is a set of directed cycles $cal(C) = {C_1, C_2, dots, C_m}$ such that every vertex $v in V$ belongs to exactly one cycle $C_i in cal(C)$.
]

A cycle cover, by definition, partitions $V$ into cycles—every vertex is covered exactly once. However, not every directed graph admits a cycle cover; for instance, a DAG has no cycles at all.

#block(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Definition 3.3 (Disjoint Cycle Cover / Cycle Packing).* A _disjoint cycle cover_ (also called _cycle packing_) of a directed graph $G = (V, E)$ is a collection $cal(C) = {C_1, C_2, dots, C_m}$ of _vertex-disjoint_ directed cycles. Unlike a cycle cover, a disjoint cycle cover need not cover all vertices. The _coverage_ of $cal(C)$ is $V(cal(C)) = union.big_(C in cal(C)) V(C)$.
]

In the TTC setting, we seek disjoint cycle covers since some patients may be structurally unable to participate in any valid exchange.

== Maximum Disjoint Cycle Cover

The first natural optimization objective is to maximize the total number of patients who receive their preferred doctor.

#block(
  width: 100%,
  inset: 1em,
  fill: luma(248),
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Problem: Maximum Disjoint Cycle Cover (MDCC)*
  
  _Input:_ A directed graph $G = (V, E)$
  
  _Output:_ A disjoint cycle cover $cal(C)$ maximizing $|V(cal(C))|$
  
  Equivalently, minimize the number of _uncovered_ vertices: $ min_(cal(C)) |V| - |V(cal(C))| $
]

#block(
  width: 100%,
  inset: 1em,
  fill: luma(248),
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Problem: $k$-Disjoint Cycle Cover (Decision Version)*
  
  _Input:_ A directed graph $G = (V, E)$ and integer $k$
  
  _Question:_ Does there exist a disjoint cycle cover $cal(C)$ such that $|V(cal(C))| >= k$?
]

== Maximum Priority Disjoint Cycle Cover

In the GP allocation setting, we distinguish between patients who currently have a GP but want a different one, and patients who have _no GP at all_ (unassigned). It is often desirable to prioritize getting unassigned patients into the system.

#block(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Definition 3.4 (Priority Partition).* Given graph $G = (V, E)$, a _priority partition_ is $V = U union.sq A$ where:
  - $U$ = _priority vertices_ (unassigned patients, i.e., $d_p = bot$)
  - $A$ = _regular vertices_ (assigned patients wanting to switch)
]

#block(
  width: 100%,
  inset: 1em,
  fill: luma(248),
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Problem: Maximum Priority Disjoint Cycle Cover (MPDCC)*
  
  _Input:_ A directed graph $G = (V, E)$ with vertex partition $V = U union.sq A$
  
  _Output:_ A disjoint cycle cover $cal(C)$ maximizing $|V(cal(C)) sect U|$
  
  That is, maximize the number of priority (unassigned) vertices covered.
]

#block(
  width: 100%,
  inset: 1em,
  fill: luma(248),
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Problem: $k$-Priority Disjoint Cycle Cover (Decision Version)*
  
  _Input:_ Directed graph $G = (V, E)$, partition $V = U union.sq A$, integer $k$
  
  _Question:_ Does there exist a disjoint cycle cover $cal(C)$ such that $|V(cal(C)) sect U| >= k$?
]

== Weighted Generalization

Both problems can be unified under a weighted formulation:

#block(
  width: 100%,
  inset: 1em,
  fill: luma(248),
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Problem: Maximum Weighted Disjoint Cycle Cover (MWDCC)*
  
  _Input:_ Directed graph $G = (V, E)$, weight function $w: V -> RR_(>= 0)$
  
  _Output:_ A disjoint cycle cover $cal(C)$ maximizing $sum_(v in V(cal(C))) w(v)$
]

The MDCC problem corresponds to $w(v) = 1$ for all $v$. The MPDCC problem corresponds to:
$ w(v) = cases(
  1 & "if" v in U,
  0 & "if" v in A
) $

A hybrid objective that prioritizes unassigned patients while still valuing coverage of assigned patients can use:
$ w(v) = cases(
  1 + epsilon & "if" v in U,
  1 & "if" v in A
) $
for some small $epsilon > 0$.

== Complexity

#figure(
  table(
    columns: 2,
    align: (left, left),
    [*Problem*], [*Complexity*],
    [MDCC on general digraphs], [NP-hard],
    [MDCC on tournaments], [Polynomial],
    [Cycle Cover existence], [NP-complete],
    [MPDCC / MWDCC], [NP-hard (reduces from MDCC)],
  ),
  caption: [Computational complexity of disjoint cycle cover problems.],
) <tab:complexity>

The standard Top Trading Cycles algorithm does not solve MDCC optimally. Instead, it employs a greedy strategy: find _any_ cycle containing the highest-priority unsatisfied patient and execute it. The choice of priority ordering—corresponding to the different `PriorityStrategy` options in our implementation—yields different heuristic approaches to these optimization problems.

== Connection to TTC Mechanism

#block(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + luma(180),
  radius: 4pt,
)[
  *Proposition 3.1 (TTC-MDCC Correspondence).* Let $I$ be a TTC instance and $G_I$ its preference graph. A disjoint cycle cover $cal(C)$ of $G_I$ corresponds to a valid set of TTC exchanges. Each cycle $C = (p_1 -> p_2 -> dots.c -> p_ell -> p_1)$ represents an exchange where patient $p_i$ receives $p_(i+1)$'s doctor (indices modulo $ell$), and all patients in the cycle are subsequently satisfied.
]

*Corollary.* Maximizing patient satisfaction in TTC is equivalent to solving the Maximum Disjoint Cycle Cover problem on the preference graph.

The key insight is that the TTC mechanism iteratively finds and executes cycles. The _order_ in which cycles are found and executed affects the final outcome when perfect coverage is not achievable. Different orderings correspond to different heuristics for the underlying optimization problem.

