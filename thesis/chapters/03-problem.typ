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

In this chapter we try to define the GP allocation problem and variations of it. In addition we go through how to construct a graph given a GP allocation problem and finally how we can reduce the GP allocation problem to the Cycle Cover problem in directed graphs.

== GP allocation problem

=== Input

Consider the GP allocation problem as given a list of patients $P$ and a list of doctors $D$. We then have patients $p_1, p_2, dots, p_n$ and doctors $d_1, d_2, dots, d_m$. We also are given the current and preferred doctor assignments as arrays:
- $D_"current" [i]$ denotes the index of the current doctor assigned to patient $p_i$ (or $bot$ if unassigned)
- $D_"preffered" [i]$ denotes the index of the preferred doctor for patient $p_i$

In addition we are given a priority function $R : P arrow NN$, mapping some positive integer to each patient, the higher the number the higher the priority.

=== Feasible solution
A feasible solution for the GP allocation problem is a subset of patients $S subset.eq P$ such that the directed graph
$
G_S = (S, E_S), E_S = {(a,b) | a, b in S, D_"preffered" ["idx"(a)] = D_"current" ["idx(b)"]}
$
consists of a vertex-disjoint union of directed cycles that cover all vertices in $S$.
Where $"idx"(x)$ is a function giving the index of a patient $x$.
So this means our solution $S$ is a set of all patients that can exchange doctor in one or more cycles.

=== Optimization criterion
With this feasible solution defined we can start defining variants where we want feasible solution that maximizes some _score_.
When choosing what patients should be exchanging we might have different qualities that we want in our solutions. 
We might want a solution that exchanges as many patients as possible or that exchanges patients with the highest priority.

First we define our general ordering
#definition("Optimization criterion")[
  An ordering $succ$ over feasible solutions. A solution is optimal if it is maximal under $succ$.
]

Then we can build upon this to create our two variants
==== Lexicographic maximization by priority
#definition("Sorted priority vector")[
  Given a feasible solution $S subset.eq P$ the sorted priority vector is:
  $
  pi(S) = "sort"((R(p)_(p in S)))
  $
  where the vector is sorted in decreasing order.
]

This vector we can then use to compare solutions and order them lexicographically by priority.

#definition("Lexicographic ordering by priority")[

  $
  S succ_"lex" S' "iff" pi(S) "is lexicographically larger than" pi(S')
  $

  In addition if $pi(S')$ is a strict prefix of $pi(S)$ then $pi(S)$ is greater.
]

We can then using this, if given two solution $S$ and $S'$ determine which one is _better_ by the lexicographic ordering py priority.

#example("Ordering two solutions lexicographically by priority")[

  We use the example graph $G$ below as the graph to create solutions from.
  
  Given solutions
  + $S = {4,5}$ represents the subgraph $G_S$ containing cycle $P_4 arrow P_5 arrow P_4$
  + $S' = {1,2,3}$ represents the subgraph $G_S'$ containing cycles $P_1 arrow P_2 arrow P_3 arrow P_1$

  For simplicity's sake we say that the $R(x) = "idx"(x)$, so in S the priority values are the same as S.

  Then $pi(S) = [5,4]$ and $pi(S') = [3,2,1]$ and we can see that $S$ is then greater by the ordering $succ_"lex"$.
  Because at the first point they differ, $S[0] != S[1]$, $S[0]$ is greater.
]

So for our first variant we want to find the maximal solution under the $succ_"lex"$ ordering.
This means always prioritizing those with highest priority.

==== Maximizing cardinality

The other variant is finding a solution with the largest cardinality, e.g. the solution that contains the most patients.

#definition("Ordering by cardinality")[
  $
  S succ_"size" S' "iff" |S| > |S'|
  $
]

This solution will be the one that exchanges the most patients and therefore makes the most amount of people happy.

#example("Ordering two solutions by cardinality")[

  Say we have graph G as in the figure below. Then our solution sets $S, S' subset.eq V(G)$

  Given solutions
  + $S = {4,5}$ represents the subgraph $G_S$ containing cycle $P_4 arrow P_5 arrow P_4$
  + $S' = {1,2,3}$ represents the subgraph $G_S'$ containing cycles $P_1 arrow P_2 arrow P_3 arrow P_1$

  By then the ordering $succ_"size"$ $|S| = 4, |S'| = 3$, so $S$ is greater than $S'$ under $succ_"size"$.
  The absolute maximal solution under $succ_"size"$ would be ${1,2,3,4,5}$ since that solution would contain both cycles.
]

#include "../figs/example-graph.typ"

#include "../figs/pareto-inefficient.typ"