#import "@preview/ctheorems:1.1.3": *
#let theorem = thmbox("theorem", "Theorem", fill: rgb("#eeffee"))
#let proof = thmproof("proof", "Proof")

= Implementation <ch:implementation>

In this chapter we present the algorithms we have implemented.
We have implemented five algorithms in total.
Three of them are exact algorithms based on cycle cancelling, _Cycle Cancelling for cardinality_, _Cycle Cancelling for utility_ and _Cycle Cancelling for strict priority_, one for each of the orderings defined in @ch:problem.
By exact we mean that the algorithm is guaranteed to return an optimal solution under its ordering.
In addition we have implemented _Greedy DFS_, a fast heuristic, and the TTC of Huitfeldt et al. @NBERw32458 that we compare against.
For each algorithm we first motivate it, then describe how it works, and finally analyze its running time.
For the exact algorithms we also give, or refer to existing, proofs of why they are exact, and show that they run in polynomial time for our priority functions.

== Different graph representations used
The algorithms use different graph representations of the GP allocation problem.
The reason is that the different orderings from @ch:problem care about different aspects of the problem.
For cardinality only the number of patients wanting each switch matters, so identical requests can be collapsed together.
For the priority based orderings each patient must be kept apart, since each patient carries their own priority.
The heuristic instead needs the patients and GPs as explicit nodes to search through.
We therefore start by defining the three representations, for each we give the formal definition first and then explain it.
Throughout, let $I = {0, ..., |P|-1}$.

=== Patient and GP graph <patient-and-gp-graph>

$
G = (V, A), quad V = P union D\
A = {(p_i, D_"pref" [i]) | i in I} union {(D_"cur" [i], p_i) | i in I}
$

The _Patient and GP graph_ contains both patients and GPs as vertices.
An edge $p_i arrow D_"pref" [i]$ means that patient $p_i$ has that GP as their preferred GP, and an edge $D_"cur" [i] arrow p_i$ means that the GP currently has $p_i$ as one of their patients.
Edges always go between a patient and a GP, never between two patients or two GPs, so the graph is bipartite with patients on one side and GPs on the other.
A directed cycle in this graph alternates between patient and GP nodes and corresponds to a valid exchange, each patient in the cycle moves to their preferred GP, and each GP loses one patient and gains one.
This representation is used by _Greedy DFS_ and by the TTC of Huitfeldt et al., both of which search for cycles directly among the patients and GPs.

=== GP collapsed graph <gp-graph-collapsed-edges>

$
G = (V, A), quad V = D \
A = {(a,b) | exists i in I " s.t. " D_"cur" [i] = a and D_"pref" [i] = b} \
u(a, b) = |{i in I | D_"cur" [i] = a and D_"pref" [i] = b}| \
c(a, b) = -1 \
$

In the _GP collapsed graph_ only the GPs are nodes.
An edge $d_a arrow d_b$ means that there exists at least one patient who currently has GP $d_a$ and has GP $d_b$ as their preferred GP.
All patients wanting the same switch are collapsed into one edge, the capacity $u(a, b)$ counts how many they are, and the cost of every edge is $-1$.
What we gain with this representation is a small graph, the number of nodes is the number of GPs and identical requests share one edge.
What we lose is the identity of the individual patients, the graph only knows how many patients want each switch, not which ones.
This is exactly the information needed for the cardinality ordering, and this representation is used by _Cycle Cancelling for cardinality_.

=== GP multigraph <gp-multigraph>

$
G = (V, A), quad V = D\
A = { (D_"cur" [i], D_"pref" [i], i) | i in I }\
u(a, b, i) = 1
$

The _GP multigraph_ is much like the _GP collapsed graph_, but instead of collapsing all equal preferences we make each preference into its own edge.
This makes the graph a multigraph, as two patients wanting the same switch give two parallel edges.
Each edge then represents exactly one patient, identified by the index $i$, and has capacity one.
Compared to the _GP collapsed graph_ we get more edges, but we keep the identity of each patient, which the priority based orderings need.
For the _Cycle Cancelling for utility_ algorithm we additionally give each edge a cost, $c(a, b, i) = -R(p_i)$, encoding the priority of the patient in the edge weight.
The _Cycle Cancelling for strict priority_ algorithm uses the same graph but without costs.

== Huitfeldt et al. TTC
To have a fair baseline to compare against we also implemented the TTC of Huitfeldt et al. @NBERw32458, described in @ch:background.
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

The motivation behind _Greedy DFS_ is to have a fast and simple heuristic that still respects priorities.
It is inspired by the TTC of Huitfeldt et al., which preserves the TTC properties at each round by restricting each GP node to a single outgoing edge @NBERw32458.
This restriction means their algorithm can miss exchanges, a cycle may exist through a GP's panel without going through the single patient the GP currently points to.
We relax this constraint, allowing each GP to keep outgoing edges to all of its current patients at the same time, and resolve ties by patient priority.
This lets the algorithm find cycles that the TTC structure misses, while still favoring high priority patients.
As we will see, the greedy choices do not guarantee an optimal solution, so _Greedy DFS_ is a heuristic, and it serves as a fast point of comparison for the exact algorithms.

The algorithm runs on the _Patient and GP graph_ defined in @patient-and-gp-graph.
Recall that a directed cycle in this graph corresponds to a valid exchange.
The idea of the algorithm is simple, it goes through the patients in decreasing priority order, and for each patient it runs a depth first search that tries to find a cycle through that patient.
If a cycle is found it is resolved immediately, every patient on the cycle moves to their preferred GP and is removed from the graph.
When the search reaches a GP node with several current patients, it explores the highest priority patient first.

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
returns a cycle if it reaches $p$ again, and returns none if no such cycle exists. When a
cycle is found, every patient in it is added to the resolved set and is not
considered again.

The greedy rule makes sure high priority patients are preferred when forming
cycles, but it does not guarantee an optimal solution. The following
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
three patients.
The greedy solution ${p_4, p_2}$ is suboptimal under both orderings, the alternative ${p_4, p_1, p_3}$ contains more patients, so it is greater under $succ_"size"$, and it contains $p_3$ which outranks $p_2$, so it is also greater under $succ_"lex"$.
The greedy choice at $d_2$ was locally motivated by priority but globally suboptimal.
This motivates the exact algorithms in the following sections.

=== Runtime
In the worst case a single DFS can visit every GP node and every patient edge before finding a cycle or giving up, costing $O(|D| + |P|)$ time.
We start one DFS per switching patient, so the total runtime is $O(|P| (|D| + |P|))$ in the worst case.
In practice the algorithm is much faster than this bound suggests, for two reasons.
First, when a DFS finds a cycle, every patient on the cycle is resolved and removed, so successful searches pay for several patients at once.
Second, when a DFS exhausts the options of a patient without finding a cycle, that patient cannot be part of any cycle in the current graph, we mark such patients as stuck and skip them in all later searches the same day.
Each day consists of one DFS per unresolved switching patient, so without this pruning a hopeless patient could be re-explored by every one of these searches.
With it, each failed patient is explored at most once per day.
The marks are cleared when the graph changes, that is, on the next day of the simulation.

== Cycle cancelling for cardinality 
This algorithm finds an optimal solution under $succ_"size"$, that is, a solution resolving the largest possible number of patients.
The motivation for this ordering is the system view, every resolved patient is one person leaving the waitlist, so resolving as many as possible each day is the most direct way of keeping the waitlists small.
The drawback is that all patients count equally, the algorithm has no notion of who has waited longest, so individual patients may be passed over day after day.

We can state this variant of the GP allocation problem as a minimum cost circulation problem.
We use the _GP collapsed graph_ defined in @gp-graph-collapsed-edges, where every edge has cost $-1$ and the capacity counts the patients wanting that switch.
Recall from @cycle-cancelling that with all costs $-1$, a circulation with minimum cost is equivalent to a circulation with the most flow, and every unit of flow here is one resolved patient.
So we find a minimum cost circulation $f^*$ using _Cycle Cancelling_, and then read the solution off the flow, for each edge carrying $k$ units of flow we resolve $k$ of the patients wanting that switch.

#pseudocode-list(booktabs: true, title: smallcaps[CycleCancellingCardinality($G$, $P$)])[
  + $"resolved" = nothing$
  + $f = $ zero circulation on $G$
  + $f^* = "CycleCancelling"(G, f)$
  + *for each* edge $(v, w) in G$ *do*
    + *if* $f^*(v, w) > 0$ *then*
      + let $S = { p in P | D_"cur" [p] = v and D_"pref" [p] = w }$
      + pick any $f^*(v, w)$ patients from $S$ and add them to $"resolved"$
    + *end*
  + *end*
  + *return* $"resolved"$
]

Because we choose edges based only on the capacity, e.g. the number of patients wanting that switch, we do not know which $k$ patients to actually resolve when we find out that $k$ patients can switch along an edge.
This choice is arbitrary since all patients in this representation are equally important and it does not affect the cardinality of the solution.
While we could choose random patients, the better choice might be to choose the $k$ patients who have the highest priority among those who want the swap. 

=== Runtime

The running time of _Cycle Cancelling_ as mentioned is $O(n m^2 C U)$ making the algorithm pseudo polynomial based on $C$, the maximum cost, and $U$, the maximum capacity.
In the _GP collapsed graph_ the maximum cost, $C$, is one, while the maximum capacity, $U$, is at most the number of patients. 
So the running time of _Cycle Cancelling for cardinality_ is polynomial in terms of the input $G, P$ as the running time is $O(n m^2 |P|)$.

Because of the _Negative Cycle Optimality Theorem_ the _Cycle Cancelling_ algorithm will not finish until there are no more negative cycles in the residual graph. 
Since the cost of each edge in the original graph is $-1$ and _Cycle Cancelling_ minimizes the total cost, the algorithm finds the optimal solution under $succ_"size"$.

== Cycle Cancelling for utility
This algorithm finds an optimal solution under $succ_"util"$.
The motivation for this ordering is to sit between the two extremes.
We want to help patients who have waited a long time, but unlike the strict priority case, a single patient with a long wait should not always outweigh a group of patients with shorter waits.
Under $succ_"util"$ every patient counts, weighted by their priority, and the base $k$ of the priority function from @ch:problem tunes how strongly long waits are favored.

The construction follows the same idea as for cardinality, but now the cost of an edge must carry the priority of its patient, so identical requests can no longer share one edge.
We use the _GP multigraph_ defined in @gp-multigraph, with the priority costs.
Each patient $i$ is their own edge in $G$ with capacity one and cost $-R(p_i)$.
A circulation then picks a feasible set of patients, and its cost is the negative of the total utility of that set, so a minimum cost circulation is an optimal solution under $succ_"util"$.
As before we find a minimum cost circulation with _Cycle Cancelling_, and every edge carrying flow corresponds to a resolved patient.

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

The running time differs from the cardinality case because the graph parameters are
different. The maximum capacity $U$ is now $1$, since every patient is its own
capacity-$1$ edge, while the maximum cost $C$ is $max R(p)$. The running time thus becomes $O(n m^2 max R(p))$. This is polynomial as long as
the priorities are polynomially bounded in the input size.

For the strict lexicographic case, the priority function is $R(p_i) = 2^i$, which
grows exponentially in the number of patients. With this priority function $C$ is
exponential in $n$, so running this algorithm on it would no longer be polynomial.
This is why we treat the strict lexicographic case separately.

There is also a practical limit on how large the priorities can get.
In our implementation the costs are stored as 128 bit integers, and with $R(a) = k^("days waiting")$ the weights grow exponentially with waiting time.
To keep the weights within bounds we divide the number of days by $10$ before exponentiating, so patients are grouped into priority buckets of $10$ days, and we cap the exponent so the weights never overflow.
For the fastest growing base we use, $k = 1.9$, the cap is reached after $1010$ days of waiting.
Past this point all longer waits get the same weight and the ordering between them is lost.
So in practice the utility algorithm with exponential priorities can only separate patients up to a bounded waiting time, this limits how long simulations we can run it on.

== Cycle Cancelling for strict priority 
The _Cycle Cancelling for strict priority_ finds an optimal solution under $succ_"lex"$.
The motivation for this ordering is fairness towards those who have waited the longest.
The algorithm never passes over a satisfiable patient in favor of anyone with lower priority, the patients who have waited the longest are helped first whenever helping them is possible at all.
The cost of this guarantee is that a single high priority patient can block several lower priority patients, so we expect it to resolve fewer patients per day than the cardinality variant.

It uses the _GP multigraph_ defined in @gp-multigraph, but without the priority costs.
The two exact algorithms for priority thus solve their problems on the same graph, _Cycle Cancelling for utility_ encodes priority in the edge weights while _Cycle Cancelling for strict priority_ encodes it in the order patients are processed.
It is based on _Cycle Cancelling_, but modifies it.
We loop through the patients in descending order of priority.
For each patient we check if their edge can be part of a cycle in the residual graph, if it can we push flow around that cycle and add the patient to the solution.
If the patient's edge already carries flow, from being part of a cycle routed by an earlier, higher priority patient, the patient is simply added to the solution.
Either way the patient's edge is then deleted from the graph, so that no later patient can undo the decision.
The algorithm is as follows:

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

Note that when a cycle is cancelled in line 9, only patient $p$ is added to the solution, not the other patients whose edges lie on the path $Q$.
Those patients are not forgotten, their edges now carry flow, so they are added to the solution when their own turn comes in the loop, by the first branch in line 6.
The flow on a routing edge can still be rerouted by a later patient, a patient is only finally committed at their own turn.

The final solution is the optimal solution under $succ_"lex"$.
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
Because one high priority patient is "better" in terms of $succ_"lex"$ than all patients with lesser priority combined, the optimal solution under $succ_"lex"$ must contain patient $p$ whenever $p$ can be satisfied together with the already committed patients.
This is exactly when the algorithm finds a cycle through $e$, so the algorithm resolves $p$ if and only if the optimal solution contains $p$.
By deleting $e$ and its residual edge after committing, no later patient can reroute the flow off of $e$, so the decision for $p$ is final.
If the algorithm does not find a cycle through $e$, then by @monotonicity-lemma no later step could have satisfied $p$ either, so deleting $e$ loses nothing.
Repeating this argument for every patient in decreasing priority order gives that the returned solution agrees with the optimal solution under $succ_"lex"$ at every position, so they are equal.

Note that with tied priorities the algorithm returns the optimal solution under its fixed tie-break order, as defined in @ch:problem.
This is not necessarily the solution satisfying the most patients within each priority level, one tied patient committed early can block several others of the same priority.
Which of the tied patients does the blocking is decided by the tie-break order, so two different tie-break orders can give solutions of different sizes, both equally valid under $succ_"lex"$ with their respective orders.

=== Runtime
As we loop over all patients we have $|P|$ iterations.
For each iteration we search for a path from $D_"pref" [i]$ to $D_"cur" [i]$ in the residual network.
This is a single graph search, for instance a BFS, and takes $O(n + m)$ time.
So the _Cycle Cancelling for strict priority_ has a running time of $O((n + m) |P|)$ and is not dependent on the maximum priority as in the _Cycle Cancelling for utility_.
Note that this is also faster than searching for negative cycles with Bellman-Ford, which would cost $O(n m)$ per patient.
We can drop the costs entirely because the priority order is already enforced by the order we process patients in, the edge weights play no role in this algorithm.

== Properties of the algorithms
In @ch:background we saw that TTC satisfies strategy proofness, individual rationality and Pareto efficiency.
A natural question is which of these properties our algorithms keep.

Individual rationality holds for all our algorithms.
A patient is either moved to the GP they reported as preferred, or stays at their current GP, no patient is ever moved somewhere they did not ask for.
So no patient is made worse off by participating.

All our algorithms also return solutions that cannot be extended.
Greedy DFS and the cycle cancelling algorithms only stop when no further cycle exists in their graph, so no additional patient can be satisfied without removing another from the solution.
In this sense the solutions are Pareto efficient, we cannot improve one patient without worsening another.
Note that this is a weaker statement than for classical TTC, where every agent has a full preference list and Pareto efficiency is over all preference profiles, in our setting a patient has only two outcomes, satisfied or not.

For strategy proofness we have to be more careful.
In a single run, a patient reports only one preferred GP.
Reporting anything other than the true preference can only result in being moved to a GP the patient does not want, or not being moved at all, so truthful reporting is a dominant strategy in the one shot setting.
In the repeated setting the situation is less clear.
Huitfeldt et al. @NBERw32458 show that when the mechanism is run repeatedly and priorities depend on waiting time, TTC style mechanisms lose strategy proofness, patients can gain by timing their requests, and some patients can end up worse off than under a first come first served system.
The same concerns apply to our algorithms, our priorities are based on days waited, so a patient could in principle time their request to affect their priority.
We have not analyzed strategic behaviour in the repeated setting and leave it as future work.
