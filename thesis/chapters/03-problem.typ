#import "@preview/ctheorems:1.1.3": *
#show: thmrules.with(qed-symbol: $square$)
#let theorem = thmbox("theorem", "Theorem", fill: rgb("#eeffee"))
#let corollary = thmplain(
  "corollary",
  "Corollary",
  base: "theorem",
  titlefmt: strong
)
#let definition = thmbox("definition", "Definition", inset: (x: 1.2em, top: 1em))

#let example = thmplain("example", "Example").with(numbering: none)
#let proof = thmproof("proof", "Proof")

= Problem Formalization <ch:problem>

In this chapter we define the GP allocation problem and also some variations of it. In addition we go through how to construct a graph given a GP allocation problem and finally how we can reduce the GP allocation problem to the Cycle Cover problem in directed graphs.

== The GP allocation problem

=== Input

In the GP allocation problem we are given a set of patients $P = {p_1, p_2, dots, p_n}$ and a set of doctors $D = {d_1, d_2, dots, d_m}$. 
We are also given the current and preferred GP assignments, for each patient:
- $D_"cur" [i]$ denotes the index of the current doctor assigned to patient $p_i$ (or $bot$ if unassigned)
- $D_"pref" [i]$ denotes the index of the preferred doctor for patient $p_i$

In addition we are given a priority function $R : P arrow NN$, mapping some positive integer to each patient, with higher numbers indicating a higher priority to help this patient.

=== Feasible solution
A feasible solution for the GP allocation problem is a subset of patients $S subset.eq P$ such that the directed graph
$
G_S = (S, E_S), E_S = {(a,b) | a, b in S, D_"pref" ["idx"(a)] = D_"cur" ["idx"(b)]}
$
consists of a vertex-disjoint union of directed cycles that cover all vertices in $S$.
This means that $S$ is a set of patients that can all get their preferred doctor if we allow for simultanous exchanges following cycles.

=== Optimization criterion
Next we can start defining variants of feasible solutions where we want to maximize some _score_.
Examples of such a score could be to find a feasible solution with many patients or something based on priority.
We might want a solution that exchanges as many patients as possible, exchanges patients with the highest priority or some mix of these.

First we define our general ordering
#definition("Optimization criterion")[
  An ordering $succ$ over feasible solutions. A solution is optimal if it is maximal under $succ$.
]

Build on this to create our three main variants of scoring functions.
==== Lexicographic maximization by priority

#definition("Characteristic vector")[
  Let patients be ordered by priority: $p_1 >= p_2 >= dots >= p_n$ where $p_1$ has the highest priority. Given a feasible solution $S subset.eq P$, the *characteristic vector* is:
  $
  chi(S) = (b_1, b_2, dots, b_n) in {0,1}^n, quad b_i = cases(1 & "if" p_i in S, 0 & "otherwise")
  $
]

#definition("Lexicographic ordering by priority")[
  $
  S succ_"lex" S' quad "iff" quad chi(S) "is lexicographically greater than" chi(S')
  $
  That is, at the first index $i$ where $S$ and $S'$ differ, $chi(S)_i = 1$ and $chi(S')_i = 0$.
]

Intuitively, $chi(S)$ is a binary number whose most significant non-zero bit corresponds to the highest-priority patient that gets helped.
The optimal solution is then the one that maximises this binary number: always satisfying the highest-priority patient it can, then subject to that the next highest, and so on.

#example("Ordering two solutions lexicographically by priority")[

  We use the example graph in @example-graph as the graph to create solutions from.

  Given solutions
  + $S = {P_4, P_5}$ represents the subgraph $G_S$ containing the cycle $P_4 arrow P_5 arrow P_4$
  + $S' = {P_1, P_2, P_3}$ represents the subgraph $G_S'$ containing the cycle $P_1 arrow P_2 arrow P_3 arrow P_1$

  For simplicity's sake we say that $R(P_i) = i$, so $P_5$ has the highest priority.

  Ordering all five patients by priority gives $p^((1)) = P_5, p^((2)) = P_4, dots, p^((5)) = P_1$. Then:
  $
  chi(S) = (1, 1, 0, 0, 0) quad chi(S') = (0, 0, 1, 1, 1)
  $
  At the first differing position ($i=1$), $chi(S)_1 = 1 > 0 = chi(S')_1$, so $S succ_"lex" S'$.
]

So for our first variant we want to find the maximal solution under the $succ_"lex"$ ordering.
This means always prioritizing patients with highest priority.

==== Maximizing cardinality

Another natural variant is to find a solution with the largest cardinality, e.g. a solution that contains the most patients.

#definition("Ordering by cardinality")[
  $
  S succ_"size" S' "iff" |S| > |S'|
  $
]

This optimal solution will then be one that exchanges the most patients.

#example("Ordering two solutions by cardinality")[

  Using the same solutions as in the previous example:
  + $S = {P_4, P_5}$ represents the subgraph $G_S$ containing the cycle $P_4 arrow P_5 arrow P_4$
  + $S' = {P_1, P_2, P_3}$ represents the subgraph $G_{S'}$ containing the cycle $P_1 arrow P_2 arrow P_3 arrow P_1$

  Under $succ_"size"$: $|S| = 2$ and $|S'| = 3$, so $S' succ_"size" S$. However the solution is ${P_1, P_2, P_3, P_4, P_5}$, the union of both cycles.

  Notice that the same two sets are reversibly ordered under the two criteria: $S succ_"lex" S'$ (because $S$ contains the highest-priority patient $P_5$) but $S' succ_"size" S$ (because $S'$ has more patients).
]

#include "../figs/example-graph.typ"


==== Maximizing utility

Finally we formulate an ordering that is a combination of $succ_"lex"$ and $succ_"size"$.
We define the total utility to be the sum of the priorities of the patients in $S$.

We recall that $R : P arrow NN$ is the function mapping a patient to a priority.

#definition("Total utility function")[
  $
  U(S) = sum_(p in S) R(p)
  $
]

#definition("Ordering by total utility")[
  $
  S succ_"util" S' "iff" U(S) > U(S')
  $
]

This means that even though solution $S'$ contains some high priority patients, if $S$ contains enough lower priority patients then $S$ can still be a higher ranked solution under $succ_"util"$.

#example("Ordering two solutions by total utility")[
  
  We use the following solutions from @example-graph-2:
  + $S = {1,3,4}$ representing the cycle $P_1 arrow P_3 arrow P_4 arrow P_1$
  + $S' = {5,2}$ representing the cycle $P_5 arrow P_2 arrow P_5$

  This gives the following total utilities: \
  $U(S) = 1 + 3 + 4 = 8$ \
  $U(S') = 2 + 5 = 7$

  It follows that $S succ_"util" S'$. Here we see that the solution with more patients with lower maximum priority has a higher score than another with greater maximum priority.
]

#include "../figs/example-graph-2.typ"


=== Priority function
The priority function has quite a large effect on the switches we end up taking. As the priority of a patient decides if that patient will be in the solution or not for most of the algorithms, except for the exact algorithm maximizing cardinality $succ_"size"$.
This is why its important to discuss what effects the function can have on solutions when we are maximizing the ordering $succ_"util"$.

If we make the priority function exponential such that $R(a) = 2^a$ then using the ordering $succ_"util"$ becomes equal to $succ_"lex"$.
While this ordering is in terms maybe the most fair it might not always be the best for the collective good, since a high priority patient might block a lot of lower priority patients from switching.
This way we might want a priority function like $R(a) = "Days patient" a "has been waiting for a switch"$.
This way if we have the choice between patient $P_i$ who has waited for 30 days $R(P_i) = 30$ and 4 patients who have waited 40 days in total an algorithm maximizing $succ_"util"$ will choose the 4 patients.

We can adjust the priority function to prioritize higher priority by making it exponential but still under the 2^a. When we use days waited we can make $R(a) = 1.1 ^ ("Days patient a has been waiting")$
Adjusting this closer to 2 makes higher priority patients more prioritized and vice versa for lowering it.