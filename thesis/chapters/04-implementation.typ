#import "@preview/ctheorems:1.1.3": *
#let theorem = thmbox("theorem", "Theorem", fill: rgb("#eeffee"))
#let proof = thmproof("proof", "Proof")

= Implementation <ch:implementation>

In this chapter we look at each algorithm we have implemented, how we implemented them, why and runtime analysis.
We have in total implemented 4 algorithms, Greedy DFS, Cycle Cancelling for Cardinality $succ_"size"$, Cycle Cancelling for Utility $succ_"util"$ and Cycle Cancelling for Lexicographic Priority $succ_"lex"$.
For the cycle cancelling algorithms we also give, or refer to existing, proofs for why they are exact.

== Different graph representations used
In the algorithms we use different graph representations, so we start by defining the different graph representations of _The GP allocation problem_.

=== Patient and Doctor graph
First we have the easiest graph containing both patients and doctors as vertices.
The edges are directed from a patient to a doctor and from a doctor to a patient. Edges never go $"patient" arrow "patient"$ or $"doctor" arrow "doctor"$.
An edge $"patient" arrow "doctor"$ symbolizes that the patient has that doctor as its preferred doctor, and $"doctor" arrow "patient"$ symbolises that the doctor currently has that patient as one of his or hers current patients.
Observe that this graph is bipartite as we have patients on one side and doctors on the other. Now the formal definition of our graph.
Let $I = {0, ..., |P|-1}$. Then:

$
G = (V, E), quad V = P union D\
E = {(p_i, D_"pref"[i]) | i in I} union {(D_"cur"[i], p_i) | i in I}
$

=== Doctor graph collapsed edges 
In this weighted graph we condense the problem to only have doctors as nodes and edges between doctors.
An edge $"doctor a" arrow "doctor b"$ symbolises that there exists a patient that wants to switch from doctor a to doctor b, or that the patient currently has doctor a and has doctor b as preferred.
The capacity of an edge indicates the number of patients wanting that switch, while the cost is -1.
We define our graph as:

$
G = (V, E), quad V = D \
E = {(a,b), (b,a) | exists i in I " s.t. " D_"cur"[i] = a and D_"pref"[i] = b} \
u(a, b) = |{i in I | D_"cur"[i] = a and D_"pref"[i] = b}| \
u(b, a) = 0 \
c(a, b) = -1 \
c(b, a) = 1
$

=== Doctor graph priority weighted
This weighted graph is much like the Doctor graph collapsed edges, but while that graph representation focuses on number of patients wanting a switch this graph focuses on the priority of each patient.
Instead of collapsing all preferences that are equal here we make each preference into an edge and weight it with the priority of that patient.

$
G = (V, E), quad V = D\
E = { (D_"cur"[i], D_"pref"[i], i) | i in I }\
u(a, b, i) = 1 \
c(a, b, i) = - R(P_i)
$


== Greedy DFS

We first consider a greedy approach inspired by the Top Trading Cycles implementation of Huitfeld et al. Their algorithm preserves TTC properties at each round by restricting each doctor node to a single outgoing edge. We relaxed this constraint, allowing each doctor to maintain outgoing edges to all of its current patients simultaneously, and resolved ties by patient priority.

We are given the lists of patients and doctors $P, D$, the assignment arrays $D_"cur", D_"pref"$, and a priority function $R : P arrow NN$, where a higher number means higher priority. We model the problem as a bipartite directed graph over patients and doctors. Let $I = {0, ..., |P|-1}$. Then:

$G = (V, E), quad V = P union D, quad E = {(p_i, D_"pref"[i]) | i in I} union {(D_"cur"[i], p_i) | i in I}$

Each patient $p_i$ has an outgoing edge to their preferred doctor, and each doctor has outgoing edges to all of its currently registered patients. A directed cycle in $G$ necessarily alternates between patient and doctor nodes and corresponds to a valid exchange: each patient in the cycle moves to their preferred doctor, and each doctor loses exactly one patient while gaining one.

The algorithm processes patients in decreasing priority order. For each patient, a DFS attempts to find a cycle through that patient; when the DFS reaches a doctor node with multiple candidate patients, it always explores the highest-priority one first.

#import "@preview/lovelace:0.3.1": *

#pseudocode-list[
  + let resolved_patients = []

  + let p_prio = Sorted list of patients by priority
  + let wants_to_switch = $["true"] * |P|$
  + *for each* $p in "p_prio"$ *do*
    + *if* let cycle = dfs_find_cycle(G, R, p) *then*
      + *for each* $p in "cycle"$ *do*
        + let $i = "idx"(p)$
        + $"wants_to_switch"[i] = "false"$
        + $"resolved_patients"."push"(p)$
      + *end*
    + *end*
  + *end*

  + *return* resolved_patients
]

The greedy rule ensures that high-priority patients are preferentially included in cycles, but it does not guarantee a globally optimal solution. The following example illustrates how the greedy choice can be locally motivated yet globally suboptimal.

#include "../figs/pareto-inefficient.typ"

In @pareto-inefficient-graph each patient $p x$ has priority $x$. The DFS begins at $p 4$ (highest priority), follows the edge to $d 2$, and there chooses $p 2$ over $p 1$ since $R(p 2) > R(p 1)$. The resulting cycle $p 4 arrow d 2 arrow p 2 arrow d 1 arrow p 4$ is committed, leaving $p 1$ and $p 3$ unsatisfied with no further cycles remaining.

Had the DFS chosen $p 1$ at $d 2$ instead, it would have found the longer cycle $p 4 arrow d 2 arrow p 1 arrow d 3 arrow p 3 arrow d 1 arrow p 4$, satisfying three patients. The greedy choice at $d 2$ was locally motivated by priority but globally suboptimal. This motivates the exact algorithms in the following sections.


/*
== Exact algorithm for maximizing total switches
#text(fill: red)[*NB: HER OG NESTE ER DET MYE AI, NOE JEG VIL HØRE DERES MENING OM. NOE AV DETTE SYNES JEG ER SKREVET BRA, ANDRE LITT USIKKER. INKLUDERT KILDENE ER IKKE RIKTIG*]

This algorithm applies the classical cycle canceling technique from network flow theory to the GP-switching problem, finding the maximum-cardinality feasible solution in polynomial time @ahuja1993[§9.6]. The key insight is that the problem reduces to a maximum integer circulation, for which efficient algorithms are known.

=== Graph structure

Rather than working with the bipartite patient-doctor graph, we compress to a weighted directed graph over doctors only. Patients who want the same switch are interchangeable: all that matters is how many can be routed. We therefore aggregate them into edge weights:

$
G = (V, E), quad V = D \
E = {(D_"cur"[i], D_"pref"[i]) | i in [|P|]} \
w(a, b) = |{i in [|P|] | D_"cur"[i] = a and D_"pref"[i] = b}|
$

A cycle $d_1 arrow d_2 arrow dots.c arrow d_k arrow d_1$ carrying $f$ units of flow corresponds to $f$ patients rotating around the cycle: each moves from their current doctor to the next in the cycle. This compression reduces the graph from $O(|P| + |D|)$ nodes to $O(|D|)$ nodes, which is significant when many patients share the same switch request.

Applying this transformation to @pareto-inefficient-graph gives the doctor graph in @pareto-inefficient-doctor-graph, where we can still identify the short cycle $d 1 arrow d 2$ and the longer cycle $d 1 arrow d 2 arrow d 3$.

#include "../figs/pareto-inefficient-doctor-graph.typ"

=== Algorithm

We want to find, for each edge $e in E$, a non-negative integer $f(e)$ representing how many of the $w(e)$ patients on that edge actually switch. Two conditions make such an assignment a valid set of simultaneous exchanges:

- *Capacity:* $0 <= f(e) <= w(e)$ for all $e in E$
- *Flow conservation:* for every doctor $d in D$: $display(sum_((v,d) in E)) f(v,d) = display(sum_((d,v) in E)) f(d,v)$

Flow conservation is exactly what forces the assignment to decompose into cycles: every doctor who loses a patient must gain one. The problem is therefore:

#block(
  stroke: 0.5pt,
  inset: 10pt,
  radius: 4pt,
  [
    *Problem: Maximum Switch Circulation*\
    \
    *Input:* A directed graph $G = (D, E, w)$ where $w: E arrow NN$\
    *Output:* An integer function $f: E arrow NN_0$ maximising $display(sum_(e in E) f(e))$ subject to:
    - $0 <= f(e) <= w(e)$ for all $e in E$
    - $display(sum_((v,u) in E) f(v,u)) = display(sum_((u,v) in E) f(u,v))$ for all $u in D$
  ]
)

This is a *maximum integer circulation* problem, solvable in polynomial time. We solve it using the *successive positive cycle* method.

Given a current flow $f$, the *residual graph* $G_f$ is built by replacing each original edge $(u, v)$ with:
- a *forward residual* arc $(u, v)$ with capacity $w(u, v) - f(u, v)$ and cost $+1$
- a *backward residual* arc $(v, u)$ with capacity $f(u, v)$ and cost $-1$

A directed cycle in $G_f$ is *positive* if the sum of its arc costs is positive, meaning it contains more forward than backward arcs, so augmenting along it strictly increases total flow. The optimality condition for maximum circulations states that $f$ is optimal if and only if $G_f$ contains no positive cycle.

The outer loop finds and augments positive cycles until none remain:

#pseudocode-list[
  + *for each* $e in E$ *do* $f(e) <- 0$ *end*
  + *while* FindPositiveCycle$(G_f)$ returns a cycle $C$ *do*
    + $b <- display(min_(e in C)) r_f (e)$ #h(2em) ▷ bottleneck residual capacity
    + *for each* $(u, v) in C$ *do*
      + $f(u,v) <- f(u,v) + b$; update residual capacities in $G_f$
    + *end*
  + *end*
  + *return* $f$
]

Positive cycles are detected using a variant of Bellman–Ford (SPFA). All nodes are seeded with distance $0$, simulating a virtual super-source at zero cost so every cycle is reachable. Edges are relaxed greedily in the maximising direction; a node relaxed $|D|$ or more times must lie on a positive cycle.

#pseudocode-list[
  + *for each* $v in D$ *do* $"dist"[v] <- 0$; $"pred"[v] <- bot$; enqueue $v$ *end*
  + *while* queue not empty *do*
    + $u <- $ dequeue
    + *for each* $(u, v)$ in $G_f$ with $r_f (u, v) > 0$ *do*
      + *if* $"dist"[u] + 1 > "dist"[v]$ *then*
        + $"dist"[v] <- "dist"[u] + 1$; $"pred"[v] <- u$
        + *if* $v$ has been enqueued $>= |D|$ times *then*
          + Walk back $|D|$ steps from $v$ along pred to find node $c$ on the cycle
          + Collect and *return* the cycle starting and ending at $c$
        + *end*
        + Enqueue $v$ if not already in queue
      + *end*
    + *end*
  + *end*
  + *return* $nothing$ #h(2em) ▷ no positive cycle; $f$ is optimal
]

#theorem("Optimality of MaxSwitchCirculation")[
  The algorithm returns a flow $f$ achieving the maximum possible value $display(sum_(e in E)) f(e)$, equal to the maximum number of patients that can simultaneously switch to their preferred doctor.
]

#proof[
  The algorithm terminates when FindPositiveCycle returns $nothing$, meaning $G_f$ contains no positive-cost directed cycle. By the cycle optimality criterion for maximum circulations: a feasible integer circulation $f$ is optimal if and only if its residual graph $G_f$ contains no positive-cost cycle. Since each forward arc carries cost $+1$ and each backward arc cost $-1$, any positive-cost cycle in $G_f$ would strictly increase $sum f(e)$; the absence of such cycles certifies that $f$ is maximum.
]

*Termination and complexity.* Each call to FindPositiveCycle runs SPFA over $|D|$ nodes and at most $|D|^2$ edges in $O(|D|^3)$ time. Each augmentation round increases total flow by at least $1$, so the number of rounds is at most $W = display(sum_(e in E)) w(e)$, the total number of patients wanting to switch, giving a worst-case bound of $O(W dot |D|^3)$. In practice each round pushes a bottleneck of $b >= 1$ units, so the number of distinct rounds is bounded by the number of edges that become saturated, at most $|E| <= |D|^2$. This gives the graph-size bound $O(|D|^5)$, independent of the patient count.

== Exact algorithm for maximizing priority

This algorithm finds the lexicographically maximal feasible solution under $succ_"lex"$, as defined in @ch:problem. It builds on the same residual-graph framework but processes patients in priority order, greedily committing each one as soon as a cycle is found. The correctness of this greedy strategy follows from the theorem below.

#theorem("Greedy choice property")[
  Let $p^*$ be the highest-priority patient that belongs to some feasible solution. Then every lexicographically maximal solution contains $p^*$.
]

#proof[
  Suppose $S^*$ is a lexicographically maximal feasible solution with $p^* in.not S^*$. By hypothesis there exists a feasible solution $S'$ with $p^* in S'$. Let $k$ be the index of $p^*$ in the priority ordering $p^((1)) succ p^((2)) succ dots.c succ p^((n))$. Every patient with index $i < k$ belongs to no feasible solution by the choice of $p^*$, so $chi(S^*)_i = chi(S')_i = 0$ for all $i < k$. At index $k$: $chi(S')_k = 1 > 0 = chi(S^*)_k$. Therefore $S' succ_"lex" S^*$, contradicting the maximality of $S^*$.
]

The algorithm applies this observation iteratively: process patients from highest to lowest priority and commit each patient to a cycle as soon as one is found.

We use the same doctor graph as before: doctors are nodes, and each patient $p$ with current doctor $u$ and preferred doctor $v$ is a directed arc $a_p = u arrow v$ with capacity $1$. Each arc also has a corresponding *backward arc* $overline(a_p) = v arrow u$ with initial capacity $0$.

*Pruning.* Before processing, we compute the strongly connected components (SCCs) of the graph. Any patient whose current and preferred doctor lie in different SCCs, or in a trivial SCC of size $1$, can never be part of any directed cycle and is immediately discarded. This avoids unnecessary BFS calls and keeps the residual graph clean throughout execution.

The algorithm processes the remaining patients from highest to lowest priority. For each patient $p$ (arc $u arrow v$), exactly one of two cases applies:

- *Case 1 (primary arc available,* $"cap"(a_p) > 0$*):* $p$ has not yet been committed. We run BFS from $v$ to $u$ in the current residual graph. If a path $Pi$ exists, then $Pi$ together with $a_p$ forms a cycle, and we commit $p$:
  - The primary arc $a_p$ is consumed *permanently*: its backward arc capacity stays $0$, so no future patient can traverse it in reverse and undo $p$'s switch.
  - Each routing arc $a$ in $Pi$ is consumed normally: $"cap"(a)$ decreases by $1$ and $"cap"(overline(a))$ increases by $1$, leaving a residual that lower-priority patients may use to reroute.
  - If no path exists, $p$ is skipped.

- *Case 2 (primary arc consumed, residual active,* $"cap"(a_p) = 0$ and $"cap"(overline(a_p)) > 0$*):* $p$'s arc was already consumed as a *routing arc* in an earlier (higher-priority) patient's cycle, which created the residual $overline(a_p)$. We now *solidify* by setting $"cap"(overline(a_p)) <- 0$, preventing any lower-priority patient from traversing this residual to undo $p$'s assignment.

#pseudocode-list[
  + Compute SCCs of $G$; discard patients whose arc spans different or trivial SCCs
  + $E_"sort" <- $ remaining patients sorted by priority descending
  + *for each* patient $p$ with arc $a_p = (u arrow v)$ in $E_"sort"$ *do*
    + *if* $"cap"(a_p) > 0$ *then*
      + *if* BfsPath$(v, u)$ in residual graph returns path $Pi$ *then*
        + $"cap"(a_p) <- 0$ #h(1fr) ▷ consume primary permanently; no residual
        + *for each* arc $a in Pi$ *do*
          + $"cap"(a) <- "cap"(a) - 1$; $"cap"(overline(a)) <- "cap"(overline(a)) + 1$
        + *end*
        + Add $p$ to $S$
      + *end*
    + *else if* $"cap"(overline(a_p)) > 0$ *then*
      + $"cap"(overline(a_p)) <- 0$ #h(1fr) ▷ solidify; lock in routing commitment
      + Add $p$ to $S$
    + *end*
  + *end*
  + *return* $S$
]

*Why the primary arc has no residual:* This is the core invariant. Once patient $p$ is committed as the beneficiary of a cycle, no future patient should be able to undo it. By not creating a residual for $a_p$, no BFS can ever traverse backwards through $p$'s switch.

*Why routing arcs do have residuals:* A lower-priority patient may be able to "take over" a routing role. For example, if $p_1$ (high priority) uses $p_3$'s arc as routing, a later patient $p_2$ may form a cycle that routes through $p_3$'s arc differently, replacing it. This is valid: only the final destination of $p_1$ (whose primary arc is irrevocable) is fixed.

#theorem("Correctness")[
  The algorithm returns the lexicographically maximal feasible solution.
]

#proof[
  By induction on the priority ordering. The Greedy Choice Theorem establishes the base case: the highest-priority feasible patient $p^*$ must appear in every lex-max solution, and BFS correctly identifies whether $p^*$ is feasible, since a path from $v$ to $u$ exists in $G$ if and only if $p^*$ lies on some directed cycle.

  For the inductive step, assume all higher-priority patients have been correctly committed. The residual graph reflects these commitments: primary arcs are permanently consumed and cannot be undone, while routing residuals remain available. We claim a path from $v$ to $u$ exists in the current residual graph if and only if $p$ can be added to a feasible solution extending all committed patients. This follows from the augmenting-path argument: any feasible cycle through $p$ either avoids all committed arcs and is directly reachable in $G_f$, or shares some routing arcs whose residuals permit an equivalent rerouting. The solidify step in Case 2 maintains the invariant: once a patient's arc is confirmed as routing for a higher-priority cycle, its residual is removed so no lower-priority patient can undo the commitment.
]

*Complexity.* Sorting patients requires $O(|P| log |P|)$. The initial SCC computation runs in $O(|D| + |E|)$. Each patient requires at most one BFS over the residual graph in $O(|D| + |E|)$ time. Processing all $|P|$ patients therefore takes $O(|P| dot (|D| + |E|))$. Since $|E| <= |D|^2$, the overall complexity is $O(|P| dot |D|^2)$.




== Metaheuristics


*/