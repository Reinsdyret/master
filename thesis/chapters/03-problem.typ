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
- $D_"cur" [i]$ denotes the index of the current doctor assigned to patient $p_i$ (or $bot$ if unassigned)
- $D_"pref" [i]$ denotes the index of the preferred doctor for patient $p_i$

In addition we are given a priority function $R : P arrow NN$, mapping some positive integer to each patient, the higher the number the higher the priority.

=== Feasible solution
A feasible solution for the GP allocation problem is a subset of patients $S subset.eq P$ such that the directed graph
$
G_S = (S, E_S), E_S = {(a,b) | a, b in S, D_"pref" ["idx"(a)] = D_"cur" ["idx"(b)]}
$
consists of a vertex-disjoint union of directed cycles that cover all vertices in $S$.
Where $"idx"(x)$ is a function giving the index of a patient $x$.
So this means our solution $S$ is a set of all patients that can exchange doctor in one or more cycles.

=== Optimization criterion
With this feasible solution defined we can start defining variants where we want feasible solution that maximizes some _score_.
When choosing what patients should be exchanging we might have different qualities that we want in our solutions. 
We might want a solution that exchanges as many patients as possible, exchanges patients with the highest priority or mix of these.

First we define our general ordering
#definition("Optimization criterion")[
  An ordering $succ$ over feasible solutions. A solution is optimal if it is maximal under $succ$.
]

Then we can build upon this to create our two variants
==== Lexicographic maximization by priority

#definition("Characteristic vector")[
  Let patients be ordered by priority: $p^((1)) succ p^((2)) succ dots.c succ p^((n))$ where $p^((1))$ has the highest priority. Given a feasible solution $S subset.eq P$, the *characteristic vector* is:
  $
  chi(S) = (b_1, b_2, dots, b_n) in {0,1}^n, quad b_i = cases(1 & "if" p^((i)) in S, 0 & "otherwise")
  $
]

#definition("Lexicographic ordering by priority")[
  $
  S succ_"lex" S' quad "iff" quad chi(S) "is lexicographically greater than" chi(S')
  $
  That is, at the first index $i$ where they differ, $chi(S)_i = 1$ and $chi(S')_i = 0$.
]

Intuitively, $chi(S)$ is a binary number whose most significant bit corresponds to the highest-priority patient. The optimal solution is the one that maximises this binary number: always satisfying the highest-priority patient it can, then subject to that the next highest, and so on.

#example("Ordering two solutions lexicographically by priority")[

  We use the example graph in @example-graph as the graph to create solutions from.

  Given solutions
  + $S = {P_4, P_5}$ represents the subgraph $G_S$ containing cycle $P_4 arrow P_5 arrow P_4$
  + $S' = {P_1, P_2, P_3}$ represents the subgraph $G_S'$ containing cycle $P_1 arrow P_2 arrow P_3 arrow P_1$

  For simplicity's sake we say that $R(P_i) = i$, so $P_5$ has the highest priority.

  Ordering all five patients by priority gives $p^((1)) = P_5, p^((2)) = P_4, dots, p^((5)) = P_1$. Then:
  $
  chi(S) = (1, 1, 0, 0, 0) quad chi(S') = (0, 0, 1, 1, 1)
  $
  At the first differing position ($i=1$), $chi(S)_1 = 1 > 0 = chi(S')_1$, so $S succ_"lex" S'$.
]

So for our first variant we want to find the maximal solution under the $succ_"lex"$ ordering.
This means always prioritizing those with highest priority.

==== Maximizing cardinality

Another variant is finding a solution with the largest cardinality, e.g. the solution that contains the most patients.

#definition("Ordering by cardinality")[
  $
  S succ_"size" S' "iff" |S| > |S'|
  $
]

This solution will be the one that exchanges the most patients and therefore makes the most amount of people happy.

#example("Ordering two solutions by cardinality")[

  Using the same solutions as the previous example:
  + $S = {P_4, P_5}$ represents the subgraph $G_S$ containing cycle $P_4 arrow P_5 arrow P_4$
  + $S' = {P_1, P_2, P_3}$ represents the subgraph $G_{S'}$ containing cycle $P_1 arrow P_2 arrow P_3 arrow P_1$

  Under $succ_"size"$: $|S| = 2$ and $|S'| = 3$, so $S' succ_"size" S$. The absolute maximal solution is ${P_1, P_2, P_3, P_4, P_5}$, the union of both cycles.

  Notice that the same two sets are ordered *oppositely* under the two criteria: $S succ_"lex" S'$ (because $S$ contains the highest-priority patient $P_5$) but $S' succ_"size" S$ (because $S'$ has more patients).
]

#include "../figs/example-graph.typ"


==== Maximizing utility

Finally we can formulate an ordering that is somewhat of a combination of $succ_"lex"$ and $succ_"size"$ which is the total utility of a solution.
We define the total utility to be sum of priorities.

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

This means that even though solution $S'$ contains some high priority patients, if $S$ contains enough lower priority patients then $S$ can be more maximal under $succ_"util"$.

#example("Ordering two solutions by total utility")[
  
  We use the following solutions from @example-graph-2:
  + $S = {1,2,3,4}$ representing the cycle $P_1 arrow P_2 arrow P_3 arrow P_4 arrow P_1$
  + $S' = {4,5}$ representing the cycle $P_4 arrow P_5 arrow P_4$

  Now the total utility is as follows: \
  $U(S) = 1 + 2 + 3 + 4 = 10$ \
  $U(S') = 4 + 5 = 9$

  So $S succ_"util" S'$, here we see that its the combination of more patients with lesser priority that can make a solution be better under $succ_"util"$ than another with greater priority.
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
Adjusting this 1.1 closer and closer to 2 makes higher priority patients more prioritized and vice versa for lowering it.