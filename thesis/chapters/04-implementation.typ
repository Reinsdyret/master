#import "@preview/ctheorems:1.1.3": *
#let theorem = thmbox("theorem", "Theorem", fill: rgb("#eeffee"))
#let proof = thmproof("proof", "Proof")

= Implementation <ch:implementation>

In this chapter we look at each algorithm we have implemented, how we implemented them, why and runtime analysis.
We have in total implemented five algorithms, Greedy DFS, Cycle cancelling for cardinality, utility and for strict priority, and the TTC of Huitfeldt et al. that we compare against.
For the cycle cancelling algorithms we also give, or refer to existing, proofs for why they are exact and why they run in polynomial time.

== Different graph representations used
In the algorithms we use different graph representations, so we start by defining the different graph representations of _The GP allocation problem_.

=== Patient and GP graph <patient-and-gp-graph>
First we have the easiest graph containing both patients and GPs as vertices.
The edges are directed from a patient to a GP and from a GP to a patient. Edges never go $"patient" arrow "patient"$ or $"GP" arrow "GP"$.
An edge $"patient" arrow "GP"$ symbolizes that the patient has that GP as its preferred GP, and $"GP" arrow "patient"$ symbolises that the GP currently has that patient as one of his or hers current patients.
Observe that this graph is bipartite as we have patients on one side and GPs on the other. Now the formal definition of our graph.
Let $I = {0, ..., |P|-1}$. Then:

$
G = (V, E), quad V = P union D\
E = {(p_i, D_"pref"[i]) | i in I} union {(D_"cur"[i], p_i) | i in I}
$

=== GP graph collapsed edges <gp-graph-collapsed-edges>
In this weighted graph we condense the problem to only have GPs as nodes and edges between GPs.
An edge $"GP a" arrow "GP b"$ symbolises that there exists a patient that wants to switch from GP a to GP b, or that the patient currently has GP a and has GP b as preferred.
The capacity of an edge indicates the number of patients wanting that switch, while the cost is -1.
We define our graph as:

$
G = (V, E), quad V = D \
E = {(a,b) | exists i in I " s.t. " D_"cur"[i] = a and D_"pref"[i] = b} \
u(a, b) = |{i in I | D_"cur"[i] = a and D_"pref"[i] = b}| \
c(a, b) = -1 \
$

=== GP multigraph <gp-multigraph>
This graph representation is much like the GP graph collapsed edges, but instead of collapsing all preferences that are equal we make each preference into its own edge.
This makes the graph a multigraph, as two patients wanting the same switch give two parallel edges.
Each edge then represents exactly one patient, identified by the index $i$.

$
G = (V, E), quad V = D\
E = { (D_"cur"[i], D_"pref"[i], i) | i in I }\
u(a, b, i) = 1
$

For the _Cycle Cancelling for utility_ algorithm we additionally give each edge a cost, $c(a, b, i) = -R(p_i)$, encoding the priority of the patient in the edge weight.
The _Cycle Cancelling for strict priority_ algorithm uses the same graph but without costs.

== Huitfeldt et al. TTC
To have a fair baseline to compare against we also implemented the TTC of Huitfeldt et al., described in @ch:background.
Our implementation is a direct port of the Python code from their replication package to Rust, keeping the algorithm logic unchanged.
The replication package is not publicly available at the time of writing, the authors kindly shared it with us directly.
Porting it to Rust means all algorithms in our experiments run in the same language, so runtime comparisons are not skewed by the choice of language.

Two adaptations were needed to run it in our setting.
First, their code expects priorities where a lower number means higher priority, as their priority is a position on a waitlist.
In our setting a higher priority number means higher priority, so we reverse the order before passing patients to the algorithm.
Second, their implementation first runs a waitlist algorithm that assigns patients to GPs with free capacity before running TTC.
In our setting every GP is at full capacity, so this step never does anything and we omit it.
The TTC itself is unchanged, GPs rank their current panel members in a fixed arbitrary order followed by the waitlisted patients in priority order, exactly as in their code.

== Greedy DFS

We first consider a greedy approach inspired by the Top Trading Cycles
implementation of Huitfeldt et al. Their algorithm preserves TTC properties at
each round by restricting each GP node to a single outgoing edge @NBERw32458. We
relax this constraint, allowing each GP to keep outgoing edges to all of its
current patients at the same time, and resolve ties by patient priority.

This algorithm runs on the Patient and GP graph defined in @patient-and-gp-graph.
Recall that this graph has both patients and GPs as vertices, with an edge
$p_i arrow D_"pref"[i]$ from each patient to their preferred GP, and an edge
$D_"cur"[i] arrow p_i$ from each GP to every patient it currently has. A directed
cycle in this graph alternates between patient and GP nodes and corresponds to a
valid exchange. Each patient in the cycle moves to their preferred GP, and each GP
loses one patient and gains one.

We are also given the priority function $R : P arrow NN$, where a higher number
means higher priority. The algorithm processes patients in decreasing priority
order. For each patient it runs a depth first search that tries to find a cycle
through that patient. When the search reaches a GP node with several current
patients, it explores the highest priority patient first.

#import "@preview/lovelace:0.3.1": *

#pseudocode-list(booktabs: true, title: smallcaps[GreedyDFS($G$, $R$)])[
  + $"resolved" = [ ]$
  + $"sorted" = $ patients sorted by $R$ in decreasing order
  + *for each* $p in "sorted"$ *do*
    + $"cycle" = "dfsFindCycle"(G, R, p)$
    + *if* $"cycle" != "none"$ *then*
      + *for each* $q in "cycle"$ *do*
        + $"resolved"."push"(q)$
      + *end*
    + *end*
  + *end*
  + *return* $"resolved"$
]

The depth first search $"dfsFindCycle"(G, R, p)$ starts at patient $p$ and follows
edges through the graph. At a patient node it follows the one edge to that
patient's preferred GP. At a GP node it has several edges to choose from, one to
each current patient, and it tries them in decreasing priority order. The search
returns a cycle if it reaches $p$ again, and returns nothing if it cannot. When a
cycle is found, every patient in it is added to the resolved set and is not
considered again.

The greedy rule makes sure high priority patients are preferred when forming
cycles, but it does not guarantee a globally optimal solution. The following
example shows how the greedy choice can be locally motivated but globally
suboptimal.

#include "../figs/pareto-inefficient.typ"

In @pareto-inefficient-graph each patient $p_x$ has priority $x$. The search begins
at $p_4$, the highest priority patient, and follows the edge to $d_2$. At $d_2$ it
chooses $p_2$ over $p_1$ since $R(p_2) > R(p_1)$. The resulting cycle
$p_4 arrow d_2 arrow p_2 arrow d_1 arrow p_4$ is committed, which leaves $p_1$ and
$p_3$ unsatisfied with no further cycles remaining.

Had the search chosen $p_1$ at $d_2$ instead, it would have found the longer cycle
$p_4 arrow d_2 arrow p_1 arrow d_3 arrow p_3 arrow d_1 arrow p_4$, which satisfies
three patients. The greedy choice at $d_2$ was locally motivated by priority but
globally suboptimal. This motivates the exact algorithms in the following sections.

== Cycle cancelling for cardinality 
This algorithm finds the solution maximal under $succ_"size"$.
We use the GP graph with collapsed edges representation defined in @gp-graph-collapsed-edges.
This way each edge $(v,w,i)$ represents patients wanting to switch from GP $v$ to $w$, the cost is $-1$ and the capacity is the number of patients wanting that switch.
We then want to find a circulation $f$ with minmial cost, we use cycle cancelling to do this. 
The following is the implementation of _Cycle Cancelling for cardinality_ using _Cycle Cancelling_ as defined in section @cycle-cancelling.


#pseudocode-list(booktabs: true, title: smallcaps[CycleCancellingCardinality($G$, $P$)])[
  + $"resolved" = nothing$
  + $f = $ zero circulation on $G$
  + $f^* = "CycleCancelling"(G, f)$
  + *for each* edge $(v, w, i) in G$ *do*
    + *if* $f^*(v, w, i) > 0$ *then*
      + let $S = { p in P | D_"cur"[p] = v and D_"pref"[p] = w }$
      + pick any $f^*(v, w, i)$ patients from $S$ and add them to $"resolved"$
    + *end*
  + *end*
  + *return* $"resolved"$
]

Because we choose edges based only on the capacity, e.g the number of patients wanting that switch, we do not know which $k$ patients to actually resolve when we find out that $k$ patients can switch on an edge. This choice is arbitrary since all patients in this representation are equally important and it does not effect the cardinality of the solution. While we could choose random patients, the realistic choice would be to choose the $k$ patients who have the highest priority among those who want the swap. 

=== Runtime

The runtime of _Cycle Cancelling_ as mentioned is $O(n m^2 C U)$ making the algorithm pseudo polynomial based on $C$, the max cost, and $U$, the max capacity.
In the GP graph with collapsed edges the max cost, $C$, is one. While the max capacity, $U$, is equal to the number of patients. 
So the runtime of _Cycle Cancelling for cardinality_ is polynomial in terms of the input $G, P$ as the runtime is $O(n m^2 |P|)$

Because of the _Negative Cycle Optimaily Theorem_ the _Cycle Cancelling_ will not finish until there are no more negative cycles in the residual graph. 
Since the cost of each edge in the original graph is -1 and _Cycle Cancelling_ minimizes the total cost, the algorithm finds the solution with the highest cardinality.

== Cycle Cancelling for utility
This algorithm finds the solution maximal under $succ_"util"$.
We use the GP multigraph representation defined in @gp-multigraph, with the priority costs.
So each patient, $i$, is an edge in $G$, the capacity is one while the cost is $-R(p_i)$.
The algorithm then proceeds as follows:
#pseudocode-list(booktabs:true, title: smallcaps[CycleCancellingUtility($G$, $P$)])[
  + $"resolved" = nothing$
  + $f = $ zero circulation on $G$
  + $f^* = "CycleCancelling"(G,f)$
  + *for each* edge $(v,w,i) in G$ *do*
    + *if* $f^*(v,w,i) > 0$ *then*
      + Add $P[i]$ to $"resolved"$
    + *end*
  + *end*
  + *return* $"resolved"$
]

=== Runtime

The runtime differs from the cardinality case because the graph parameters are
different. The maximum capacity $U$ is now $1$, since every patient is its own
capacity-$1$ arc, while the maximum cost $C$ is $max R(p)$. The bound
$O(n m^2 C U)$ becomes $O(n m^2 max R(p))$. This is polynomial as long as
the priorities are polynomially bounded in the input size.

For the strict lexicographic case, the priority function is $R(p_i) = 2^i$, which
grows exponentially in the number of patients. With that priority function $C$ is
exponential in $n$, so running this algorithm on it would no longer be polynomial.
This is why we treat the strict lexicographic case separately and give it its own
algorithm.

There is also a practical limit on how large the priorities can get.
In our implementation the costs are stored as 128 bit integers, and with $R(a) = k^("days waiting")$ the weights grow exponentially with waiting time.
To keep the weights within bounds we divide the number of days by $10$ before exponentiating, so patients are grouped into priority buckets of $10$ days, and we cap the exponent so the weights never overflow.
For the fastest growing base we use, $k = 1.9$, the cap is reached after $1010$ days of waiting.
Past this point all longer waits get the same weight and the ordering between them is lost.
So in practice the utility algorithm with exponential priorities can only separate patients up to a bounded waiting time, this limits how long simulations we can run it on.

== Cycle Cancelling for strict priority 
The _Cycle Cancelling for strict priority_ finds the solution maximal under $succ_"lex"$.
It uses the GP multigraph representation defined in @gp-multigraph, but without the priority costs.
The two exact algorithms for priority thus solve their problems on the same graph, _Cycle Cancelling for utility_ encodes priority in the edge weights while _Cycle Cancelling for strict priority_ encodes it in the order patients are processed.
It is based on _Cycle Cancelling_, but modifies it.
First we loop through patients in descending order of priority, if we find any cycle with this patient in the residual graph we cancel it and add the patient to the solution.
If the patient already has flow from being in a cycle with another patient with greater priority, then we just remove it from the graph.
This way the highest priority patient is either in a cycle or not, the patient is removed from the graph at the end either way.
The algorithms is as follows:

#pseudocode-list(booktabs:true, title: smallcaps[CycleCancellingStrictPriority($G$, $R$, $P$)])[
  + $"resolved" = nothing$
  + $f = $ zero circulation on $G$
  + $"sorted" = $ $P$ sorted by $R$ in decreasing order
  + *for each* $p in "sorted"$ *do*
    + $i = "idx"(p)$, $quad e = (D_"cur"[i], D_"pref"[i], i)$
    + *if* $f(e) = 1$ *then*
      + Add $p$ to $"resolved"$
    + *else if* $exists$ directed path $Q$ from $D_"pref"[i]$ to $D_"cur"[i]$ in $G(f)$ *then*
      + Push one unit of flow on $e$ and on every edge of $Q$
      + Add $p$ to $"resolved"$
    + *end*
    + Delete $e$ and its residual edges from $G$
  + *end*
  + *return* $"resolved"$
]

The final solution is the maximal solution under $succ_"lex"$.
To show why we first need a small observation about how the feasible solutions change as the algorithm runs.

#theorem("Monotonicity of feasibility")[
  Let $F$ be the set of patient edges that currently carry flow, and let $e$ be an edge not in $F$.
  If there is no circulation that has flow on every edge of $F$ and on $e$, then there is also no such circulation after more patients have been committed.
]<monotonicity-lemma>

#proof[
  Committing a patient adds their edge to $F$ and deletes the edge from the residual network.
  Both of these only add constraints, no new circulations become possible.
  So if no circulation containing $F union {e}$ exists now, none can exist later. 
]

Recall that a path from $D_"pref" [i]$ to $D_"cur" [i]$ in the residual network $G(f)$, together with the edge $e$, forms a cycle through $e$.
By standard flow theory such a path exists if and only if there is a circulation that keeps flow on all committed edges and also has flow on $e$.
So the search in the algorithm is an exact test of whether patient $p$ can still be satisfied together with all higher priority patients that are already resolved.

Now consider the patients in the order the algorithm processes them.
Because one high priority patient is "better" in terms of $succ_"lex"$ than all patients with lesser priority combined, the maximal solution under $succ_"lex"$ must contain patient $p$ whenever $p$ can be satisfied together with the already committed patients.
This is exactly when the algorithm finds a cycle through $e$, so the algorithm resolves $p$ if and only if the maximal solution contains $p$.
By deleting $e$ and its residual edge after committing, no later patient can reroute the flow off of $e$, so the decision for $p$ is final.
If the algorithm does not find a cycle through $e$, then by @monotonicity-lemma no later step could have satisfied $p$ either, so deleting $e$ loses nothing.
Repeating this argument for every patient in decreasing priority order gives that the returned solution agrees with the maximal solution under $succ_"lex"$ at every position, so they are equal.

=== Runtime
As we loop over all patients we have $|P|$ iterations.
For each iteration we search for a path from $D_"pref" [i]$ to $D_"cur" [i]$ in the residual network.
This is a single graph search, for instance a BFS, and takes $O(n + m)$ time.
So the _Cycle Cancelling for strict priority_ has a runtime of $O((n + m) |P|)$ and is not dependent on the maximum priority as in the _Cycle Cancelling for utility_.
Note that this is also faster than searching for negative cycles with Bellman-Ford, which would cost $O(n m)$ per patient.
We can drop the costs entirely because the priority order is already enforced by the order we process patients in, the edge weights play no role in this algorithm.

