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

In this chapter we define the GP allocation problem, what a feasible solution to it is, and three variants of what an optimal solution is.
Note that in @cycle-cancelling the costs were placed on the edges of a graph, while in this chapter the priorities are placed on the patients.
The two are connected, in @ch:implementation we construct graphs for the GP allocation problem where each patient corresponds to an edge, and the priority of the patient becomes the cost of that edge.

== The GP allocation problem

=== Input

In the GP allocation problem we are given a set of patients $P = {p_1, p_2, dots, p_n}$ and a set of GPs $D = {d_1, d_2, dots, d_m}$. 
We are also given the current and preferred GP assignments for each patient:
- $D_"cur" [i]$ denotes the index of the current GP assigned to patient $p_i$ (or $bot$ if unassigned)
- $D_"pref" [i]$ denotes the index of the preferred GP for patient $p_i$

In addition we are given a priority function $R : P arrow NN$, mapping some positive integer to each patient, with higher numbers indicating a higher priority to help this patient.

=== Feasible solution
#definition("Feasible solution")[
  
A subset $S subset.eq P$ of switching patients is a *feasible solution* if the patients in $S$ can take over each other's slots such that every patient in $S$ ends up at their preferred GP and the number of patients at each GP is unchanged.
Formally, define the directed graph $G_S = (S, E_S)$ where
$
E_S = { (a, b) | a, b in S, "the preferred GP of" a "is the current GP of" b }.
$
An edge $(a, b)$ means that patient $a$ can take over the slot of patient $b$.
Then $S$ is feasible if and only if $G_S$ contains a spanning subgraph that is a vertex-disjoint union of directed cycles covering every patient in $S$, that is, a selection of edges where every patient has exactly one outgoing and one incoming edge among the selected.
]
Note that a patient can have several outgoing edges in $G_S$, one to each patient in $S$ that currently has their preferred GP.
A solution consists of choosing one of them for each patient, such that the chosen edges form the cycles.

=== Optimization criterion
Next we define what it means for one feasible solution to be better than another.
We might want a solution that exchanges as many patients as possible, a solution that exchanges the patients with the highest priority, or some mix of these.
To compare solutions we use an ordering.
#definition("Optimization criterion")[
  An optimization criterion is an ordering $succ$ over feasible solutions.
  A solution is optimal if no feasible solution is greater under $succ$.
]
Each of the following variants of the GP allocation problem is defined by choosing such an ordering, and the goal of the algorithms in @ch:implementation is to find an optimal solution under the chosen ordering.
We build on this to create our three main variants.

==== Lexicographic maximization by priority

#definition("Characteristic vector")[
  Let patients be ordered by priority: $p_1 >= p_2 >= dots >= p_n$ where $p_1$ has the highest priority, with ties broken by a fixed arbitrary order, in our case the patient identifier.
  This makes the ordering a total order, so the position of each patient in the vector is well-defined.
  Given a feasible solution $S subset.eq P$, the *characteristic vector* is:
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

Note that when priorities are tied, the tie-break order matters for which solution is optimal.
Two tied patients occupy two distinct positions in the vector, and the one placed first wins any comparison between solutions that disagree on them, even if choosing the other would have allowed more patients in total to be satisfied.
So the optimal solution under $succ_"lex"$ is always defined relative to the chosen tie-break order, different tie-break orders can give optimal solutions of different sizes.

#example("Ordering two solutions lexicographically by priority")[

  We use the example graph in @example-graph as the graph to create solutions from.

  Given solutions
  + $S = {p_4, p_5}$ represents the subgraph $G_S$ containing the cycle $p_4 arrow p_5 arrow p_4$
  + $S' = {p_1, p_2, p_3}$ represents the subgraph $G_(S')$ containing the cycle $p_1 arrow p_2 arrow p_3 arrow p_1$

  For simplicity's sake we let $R(p_i) = i$, so $p_5$ has the highest priority.

  Ordering all five patients by priority gives $p^((1)) = p_5, p^((2)) = p_4, dots, p^((5)) = p_1$. Then:
  $
  chi(S) = (1, 1, 0, 0, 0) quad chi(S') = (0, 0, 1, 1, 1)
  $
  At the first differing position ($i=1$), $chi(S)_1 = 1 > 0 = chi(S')_1$, so $S succ_"lex" S'$.
]

So for our first variant we want to find the optimal solution under the $succ_"lex"$ ordering.
This means always prioritizing patients with highest priority.

==== Maximizing cardinality

Another natural variant is to find a solution with the largest cardinality, e.g. a solution that contains the most patients.

#definition("Ordering by cardinality")[
  $
  S succ_"size" S' "iff" |S| > |S'|
  $
]

An optimal solution will then be one that exchanges the most patients.

#example("Ordering two solutions by cardinality")[

  Using the same solutions as in the previous example:
  + $S = {p_4, p_5}$ represents the subgraph $G_S$ containing the cycle $p_4 arrow p_5 arrow p_4$
  + $S' = {p_1, p_2, p_3}$ represents the subgraph $G_(S')$ containing the cycle $p_1 arrow p_2 arrow p_3 arrow p_1$

  Under $succ_"size"$: $|S| = 2$ and $|S'| = 3$, so $S' succ_"size" S$. However the optimal solution is ${p_1, p_2, p_3, p_4, p_5}$, the union of both cycles.

  Notice that the same two sets are reversibly ordered under the two criteria: $S succ_"lex" S'$ (because $S$ contains the highest-priority patient $p_5$) but $S' succ_"size" S$ (because $S'$ has more patients).
]

#include "../figs/example-graph.typ"


==== Maximizing utility

Finally we formulate an ordering that is a combination of $succ_"lex"$ and $succ_"size"$.
The lexicographic ordering can let a single high priority patient outweigh any number of lower priority patients, while the cardinality ordering ignores priorities completely.
A natural middle ground is to let every patient count, weighted by their priority.
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
  + $S = {p_1, p_3, p_4}$ representing the cycle $p_1 arrow p_3 arrow p_4 arrow p_1$
  + $S' = {p_5, p_2}$ representing the cycle $p_5 arrow p_2 arrow p_5$

  As in the previous examples we let $R(p_i) = i$.
  This gives the following total utilities: \
  $U(S) = 1 + 3 + 4 = 8$ \
  $U(S') = 2 + 5 = 7$

  It follows that $S succ_"util" S'$. Here we see that the solution with more patients with lower maximum priority has a higher score than another with greater maximum priority.
]

#include "../figs/example-graph-2.typ"


=== Priority function
The priority function has quite a large effect on the final solution.
The priority function chosen will largely decide which patients will be in the final solution.
This is why it is important to investigate what effects the function can have on solutions when we are maximizing the ordering $succ_"util"$ or $succ_"lex"$.

If we make the priority function exponential, such as $R(a) = 2^a$, then the ordering $succ_"util"$ becomes the same as $succ_"lex"$.
This is because $2^a$ is larger than the sum of all lower priorities below it.
If every patient has a distinct priority, so that patient $a$ has priority $2^a$, then no group of patients with lower priority can have a total priority that reaches $2^a$, since their priorities sum to at most $2^0 + 2^1 + dots + 2^(a-1) = 2^a - 1$.
So a solution maximizing $succ_"util"$ will always pick the higher priority patient first, which is exactly what $succ_"lex"$ does.
Note that this argument requires the priorities to be distinct, two patients that share the priority $2^a$ together outweigh one patient with priority $2^(a+1)$.

While this ordering is in a sense the most fair, since we never let a patient with lower priority switch before one with greater, it might not always be the best for the collective good, since a high priority patient might block several lower priority patients from switching.
One solution to this could be a priority function where $R(a)$ depends on how long patient $a$ has been waiting for a switch.
Then if we have the choice between patient $p_i$ who has waited 30 days, $R(p_i) = 30$, and four patients who have waited 40 days in total, an algorithm maximizing $succ_"util"$ will choose the four patients.

We can also use a priority function in between these two extremes. Instead
of a fixed base we use a base $k$ with $1 <= k <= 2$ and set
$
  R(a) = k^("days patient" a "has been waiting").
$
The base $k$ controls how much higher priority patients are favored. At $k = 1$
every patient gets priority $1$, no matter how long they have waited. The total
utility $U(S)$ is then just the number of patients in $S$, so maximizing
$succ_"util"$ becomes the same as maximizing $succ_"size"$. 

At $k = 2$ the priorities grow fast enough that a higher priority patient is worth more than any
group of lower priority patients below them, so $succ_"util"$ becomes the same as $succ_"lex"$, as shown above.
This equivalence only holds exactly when all waiting times are distinct.
With this priority function many patients share the same number of days waited, so the priorities are not distinct and the argument above no longer applies.
The lexicographic ordering itself is also only defined up to tie-breaking when priorities are equal, so with ties the $k = 2$ endpoint approximates $succ_"lex"$ rather than matching it.

So the cardinality and lexicographic orderings are not separate cases but the two endpoints of the utility ordering.
This is why our exact algorithm for $succ_"util"$, which we present in @ch:implementation, also solves $succ_"size"$, by giving it the priority function with $k = 1$.
The lexicographic case at $k = 2$ is different in practice, for two reasons.
First, the priorities $2^a$ grow very large, and the runtime of cycle cancelling depends on the size of the costs, so running the utility algorithm with $k = 2$ would be slow.
Second, as noted above, with tied waiting times the $k = 2$ endpoint does not match $succ_"lex"$ exactly.
For these reasons we treat strict lexicographic priority as its own case with its own algorithm, which we describe in @ch:implementation.
That algorithm does not encode priorities in weights at all, it processes patients in priority order with ties broken by a fixed order, and so handles ties exactly.

