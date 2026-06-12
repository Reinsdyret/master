#import "@preview/ctheorems:1.1.3": *
#show: thmrules.with(qed-symbol: $square$)
#let theorem = thmbox("theorem", "Theorem", 
  base: "heading", 
  base_level: 2,
  fill: rgb("#eeffee"))
#let corollary = thmplain(
  "corollary",
  "Corollary",
  base: "theorem",
  titlefmt: strong
)
#let definition = thmbox("definition", "Definition", inset: (x: 1.2em, top: 1em))

#let example = thmplain("example", "Example").with(numbering: none)
#let proof = thmproof("proof", "Proof")


= Background <ch:background>

Problems like the GP allocation problem have been studied for quite some time with a number of variations.
In addition there are different perspectives on how to solve these problems.
For instance in economics the focus is often on the fairness of an assignment, trying to optain a stable and strategy proof assignment.
In this chapter we describe similar problems to the GP allocation problem, describe the Top Trading Cycles algorithm (TTC) and the use of this by Huitfeldt et al for the GP allocation problem @NBERw32458.
Finally we describe cycle cancelling, an optimization technique that our exact algorithms are based on.

== Related problems
There are many different variations of assignment problems.
In a typical setting there are agents and objects where the agents want to obtain particular objects.
In the GP allocation problem the patients are the agents and GPs are the objects.
The typical assignment problems are one-to-one meaning that one agent is assigned to at most one object and an object can only be "owned" by one agent.
In addition we also have many-to-one assignment problems, like the GP allocation problem where one agent can only be assigned to one object but one object can be "owned" by more than one agent.
The terminology that a patient "owns" a GP means that the patient has that GP as his or hers current GP.
It might be misleading to say that our patients "own" GPs but this makes more sense in other problems that we will describe, owning a GP in our case means having that GP as a current GP.

=== One-to-one assignment problems
Recall that in a one-to-one assignment problem each agent is assigned to at most one object and each object can be "owned" by at most one agent.
We look at two such problems, the Housing Marked and the Kidney Exchange.
Both differ from the GP allocation problem since a GP can be owned by many patients at once, but can be solved using the same algorithms.
==== Housing Market
The Housing Market model introduced in Shapley and Scarf @SHAPLEY197423 describes a model with traders (agents) and indivisible goods (objects).
These goods can be houses, making it a housing market. Each agent already has one house but may want to switch.
Every agent has a preference list ordering every house in order of preference, Shapley and Scarf combine all these preference lists to make a preference matrix @SHAPLEY197423.

It is in this article from Shapley and Scarf @SHAPLEY197423 that they also introduce the Top Trading Cycles algorithm and describe how it is fitting for models like the Housing Market.
We also see similarities to the GP allocation problem in that we have agents already owning an object but wanting to switch.
However two differences are that a GP can have multiple patients while a house can only be owned by one agent, and also that in the Housing market each agent has a complete preference list of objects while in the GP assignment problem each agent has only one preferred GP.

==== Kidney exchange
The Kidney Exchange problem describes an issue when trying to find compatible kidney donors.
It describes the case where we have pairs of patients and donors, but the patient is incompatible with its donor's kidney. 
We then need to find some way to swap around the donors to compatible patients. Either with pairwise swapping or through longer cycles.
If we visualize the problem as a directed graph, each vertex is a patient and donor pair. An edge from pair A to pair B means that donor A is compatible with patient B.
Now cycles can form as in @kidney-exchange-example where we have a cycle of length three $A arrow B arrow C$. This means that the donor in pair A can be matched with the patient in pair B and so on, assuring that each patient in the cycle is assigned a compatible donor.

#include "../figs/kidney-exchange-example.typ"

Abraham, Blum and Sandholm @ABRAHAM2007 show that, with unbounded cycle length, a maximum-weight exchange can be found in polynomial time @enwiki:1351916222.
An issue often arising with unrestricted cycle length is if a donor suddenly refuses to donate. If this happens after their patient has received a kidney then a patient is left without a donor and cannot exchange later.
Because of this the problem is often considered with a restricted cycle length to allow all operations to be executed at the same time. This way no donor can refuse in the middle.
For each pair in a cycle you need 2 operations, for a cycle of length $k$ you need $2k$ operations at the same time. 
It is for this reason that the Kidney Exchange problem often is considered with a max cycle length $k$, as the number of operating rooms in a state/city is limited.
Finding a maximum-cardinality exchange with cycles of length at most $k$, for any fixed $k >= 3$, is an NP-hard computational problem @ABRAHAM2007 @enwiki:1351916222.

The Kidney Exchange problem and the GP allocation problem have much in common.
In both problems agents already have an object, but want to switch for another.
We need to find cycles to exchange the objects in such a manner that as many patients as possible are satisfied.
The key difference is that only one donor can be "owned" by one patient while in the GP allocation problem a GP can be "owned" by multiple patients.

=== Many-to-one assignment problems
In a many-to-one assignment problem each agent is still assigned to at most one object, but an object can be "owned" by more than one agent.
This is the same structure as the GP allocation problem, where a GP can have many patients. We look at one such problem, the College Admissions problem.
==== The College Admissions problem 
The College Admissions problem, introduced by Gale and Shapley @GALES1962, describes the problem of assigning applicants to colleges.
Each college has a certain quota, i.e. a maximum number of applicants that can be admitted and each applicant ranks the colleges in order of preference.
An applicant does not have to rank every college and can skip colleges he or she does not want to attend @GALES1962.

The problem is then to find a stable assignment, meaning that there does not exist a case where there are two applicants $alpha$ and $beta$ who are assigned to colleges $A$ and $B$, although $beta$ prefers $A$ to $B$ and $A$ prefers $beta$ to $alpha$.
Such a pair would break the assignment, since both would rather be
matched to each other.

There can be many stable assignments for the same preferences, so there is not one single best assignment.
Among all stable assignments there is one that is best for every applicant at the same time, and one that is best for every college at the same time.
Gale and Shapley @GALES1962 introduce an algorithm called deferred-acceptance that finds a stable assignment in polynomial time @GALES1962.
The algorithm has one side propose and the other side accept or reject.
The side that proposes ends up with the assignment that is best for them and worst for the other side.

There are some similarities between the College Admissions problem and the GP assignment problem.
Like how applicants prefer some colleges while patients prefer a GP.
One key difference is that patients, only prefer one GP.
Another difference is that patients already have existing GPs, while applicants do not already have a college they want to switch from.

== Top Trading Cycles
The Top Trading Cycles, or TTC, is an algorithm that finds cycles of agents that can
all trade their objects at the same time. We look at it in two forms. First we
explain the classical TTC, which works on the Housing Market problem. This is the
original version and we explain it to show where the method comes from. Then we
explain the version made by Huitfeldt et al. for the GP allocation problem. It is
this version that inspired our work, and it is the one we later compare our
algorithms against.

=== Classical TTC
The Top Trading Cycles algorithm (TTC), developed by Gale and published by Shapley and Scarf @SHAPLEY197423, is an algorithm to find a re-allocation of objects, or goods, without using money.
It was introduced as a way of solving the Housing Market problem.
The algorithm finds a re-allocation of houses to traders, such that all mutually-beneficial exchanges have been realized @SHAPLEY197423.

TTC works on a directed graph.
For the Housing Market problem each agent is a node.
For an agent $i$ we let $"Top"(i)$ denote the agent holding the house that $i$ prefers the most among the houses still in the graph, if $i$'s own house is the most preferred then $"Top"(i) = i$.
The algorithm is as follows @SHAPLEY197423:
+ Insert a directed edge from each agent $i$ to $"Top"(i)$.
+ Find a cycle, which is guaranteed to exist since every agent has exactly one outgoing edge, and execute the trades in that cycle. Then remove all involved agents from the graph.
+ If there are remaining agents, repeat from step 1.

This guarantee is the pigeonhole principle, a finite graph where every node has exactly one outgoing edge must contain a cycle.
The cycle can be of length 1, when an agent's top house is its own, then the agent simply keeps their house and is removed.
For $"Top"(i)$ to always be defined the agent does not need to rank every house.
It is enough that each agent has a strict ranking of some of the houses that includes their own house.
An agent's own house stays in the graph as long as the agent does, so $"Top"(i)$ always finds a house no further down the list than the agent's own.
Houses ranked below the agent's own house can never be assigned to them, so they can be left out entirely.

We now consider an example with three agents, $A$, $B$ and $C$, owning houses $h_A$, $h_B$ and $h_C$, each with a strict preference list over the houses:
- $A = [h_B, h_C, h_A]$
- $B = [h_C, h_A, h_B]$
- $C = [h_B, h_C, h_A]$
If we make the graph where each agent points to their $"Top"$ we get @ttc-graph.
#include "../figs/ttc-graph.typ";
We have the cycle $B arrow C arrow B$, shown in @ttc-graph, so we execute the trades, $B$ gets $h_C$ and $C$ gets $h_B$, and remove both agents from the graph.
The last remaining agent is $A$, and as $h_B$ and $h_C$ are no longer in the graph, $"Top"(A)$ is now $A$ itself, shown in @ttc-graph-2.
#include "../figs/ttc-graph-2.typ"
We execute this trivial trade, $A$ keeps $h_A$, and are left with no more agents, making TTC terminate.

Roth proved that TTC is strategy proof, meaning all agents are motivated to report their true preferences @ROTH1982127.
TTC also satisfies individual rationality, that an agent is at least as well off by participating, and Pareto efficiency, that the objects are allocated such that no agent can improve without worsening another agent @SHAPLEY197423 @ROTH1982127.

Now using TTC for the GP assignment problem is challenging, but not because patients rank only two GPs.
Note that a patient's preferences are exactly a list of the minimal form above, the preferred GP followed by their current GP.
The real difference is that several patients can hold the GP that patient $i$ prefers, so $"Top"(i)$ is no longer a single agent.
How should we choose between these?
This is what Huitfeldt et al. @NBERw32458 address.
Notably they show that in their dynamic setting, where the mechanism is run repeatedly and patients care about waiting time, TTC loses some of the classical properties, it is no longer strategy proof and some patients can be left worse off than under the existing first come first served system.
Later we run their TTC for the GP assignment problem and compare it with our algorithms, but first we explain it and how it differs from the classical TTC.

=== GP assignment problem TTC
In their paper, Huitfeldt et al. @NBERw32458 study the GP assignment problem using real patient and GP data from Norway.
Before explaining their algorithm we need one piece of terminology.
The set of patients currently enrolled with a GP is called the GP's _panel_, and each GP has a cap on how many patients their panel can hold.
A GP has available capacity when their panel holds fewer patients than its cap, such a GP has open slots that a waiting patient can fill directly, without any exchange.

In their model some GPs have available capacity.
Their mechanism therefore first runs a waitlist algorithm that fills all open slots, it goes through every GP with available capacity and assigns the waiting patients with the highest priority to the open slots, until no patient is waiting for a GP with available capacity @NBERw32458.
Only after this do they run TTC on the patients that remain on waitlists.
In our setting every GP is at full capacity, so the waitlist step never assigns anyone, and in the following we explain their TTC without it.

The algorithm works on a graph with both the waiting patients and their GPs as nodes:
+ Each patient points to their preferred GP, if the preferred GP has been removed from the graph the patient points to their current GP instead. Each GP has a preference list consisting of its panel members in a fixed arbitrary order, followed by the waiting patients wanting that GP in priority order. The GP points to the first patient in this list that is still in the graph.
+ Find a cycle in the graph and resolve it, every patient in the cycle is moved to the GP they point to. Remove the resolved patients from the graph. If a GP has no panel members left in the graph, remove the GP.
+ Repeat from step 1 until there are no more patients.

Like in the classical TTC algorithm a cycle is guaranteed to exist by the pigeonhole principle, every node in the graph, patient or GP, has exactly one outgoing edge.
A cycle can also consist of just a patient and their own GP pointing at each other, then the patient simply stays with their current GP and is removed.
Huitfeldt et al. also propose a variant with adjusted priorities, TTC with priority, that protects patients whose GPs have available capacity, since in our setting no GP has available capacity we compare only against the basic TTC @NBERw32458.

== Cycle cancelling <cycle-cancelling>
Cycle cancelling is a technique used in flow and circulation problems.
In these problems each edge of a graph has a cost, and the goal is to find a circulation of flow whose total cost is as small as possible.
First we define the terms needed for cycle cancelling, then we present the technique itself.
We use definitions from Ahuja, Magnanti and Orlin @ahuja1993.

=== The minimum cost circulation problem

Let $G = (V, E)$ be a directed multigraph with $n$ vertices and $m$ edges. Each edge
is a triple $(v, w, i)$ where $v$ is the tail, $w$ is the head and $i$ is an index.
The pair $(v, w)$ gives the endpoints of the edge. The index $i$ separates edges that
have the same endpoints, so $G$ can have parallel edges. This means an edge is
identified by the full triple and not just by its endpoints.

The multigraph $G$ is a circulation network if each edge $(v, w, i)$ has a capacity
$u(v, w, i) >= 0$ and a cost $c(v, w, i) in ZZ$. For a vertex $w in V$ we let
$"in"(w) = {(v, w, i) in E}$ be the edges whose head is $w$, and
$"out"(w) = {(w, v, i) in E}$ the edges whose tail is $w$.

#definition("Circulation")[
  A circulation is a function $f$ that assigns a value $f(v, w, i)$ to each edge and
  satisfies the capacity constraints
  $
    0 <= f(v, w, i) <= u(v, w, i) quad quad forall (v, w, i) in E
  $
  and the conservation constraints
  $
    sum_((u, w, i) in "in"(w)) f(u, w, i)
      = sum_((w, v, j) in "out"(w)) f(w, v, j) quad quad forall w in V.
  $
]

The conservation constraint states that at each vertex the flow coming in equals the
flow going out. No flow enters or leaves $G$ from the outside, so all flow
circulates in the network. The cost of a circulation $f$ is
$
  "cost"(f) = sum_((v, w, i) in E) c(v, w, i) f(v, w, i).
$

#definition("Minimum cost circulation problem")[
  Given a circulation network $G$, the minimum cost circulation problem is to find
  a circulation $f$ with minimum cost.
]

Note that the costs are what make the problem interesting.
If every edge has a cost $c(v, w, i) >= 0$, then the circulation with zero flow on
every edge always has minimum cost, and the problem is trivial.
The problem becomes meaningful when some edges have negative costs.
Then the zero circulation is still feasible, but pushing flow along negative cost
edges lowers the cost, so the problem becomes to find where flow should be pushed.
In @ch:implementation we show how to construct circulation networks for the GP
assignment problem, where each edge is a possible patient switch and all costs are
negative, so that a minimum cost circulation corresponds to a best set of switches.
For example, if every edge has cost $-1$, then
$"cost"(f) = -sum_((v,w,i) in E) f(v,w,i)$, the negative of the total flow, so a
circulation with minimum cost is equivalent to a circulation with the most flow.

An example circulation network is shown in @example-circulation.
For simplicity this example is not a multigraph.
In the example each edge is labeled $x, y$, meaning the edge has cost $x$ and
capacity $y$, and every edge has cost $-1$ and capacity $1$.
The network has three feasible circulations: no flow on any edge, the cycle
$d arrow c arrow b arrow d$, and the cycle $a arrow b arrow d arrow c arrow a$.
Their costs are $0$, $-3$ and $-4$.
The longer cycle is the optimal circulation since it has the most flow and the
lowest cost.

#include "../figs/example-circulation.typ"

=== The residual network

The residual network shows what capacity is left after a circulation has been
computed, and what flow can be undone. Given a circulation network $G$ and a
circulation $f$, we build the residual network $G(f)$ edge by edge. Each edge
$(v, w, i) in E$ gives two residual edges.

- A forward edge $(v, w, i)$ with cost $c(v, w, i)$ and residual capacity
  $u_f (v, w, i) = u(v, w, i) - f(v, w, i)$.
- A backward edge $(w, v, i)$ with cost $-c(v, w, i)$ and residual capacity
  $u_f (w, v, i) = f(v, w, i)$.

A residual edge is in $G(f)$ only if its residual capacity is greater than $0$. An
edge with residual capacity $0$ is saturated. We build the residual edges from each
edge $(v, w, i)$ and not from the pair $(v, w)$, so the index $i$ is kept on both
residual edges. This means that $G(f)$ is also a directed multigraph and every
residual edge is still uniquely identified.

The two kinds of residual edges have different roles. A forward edge has the same
cost as its original edge, and pushing more flow through it increases $f$ on that
edge. A backward edge has the negated cost, and pushing flow through it decreases
$f$ on the original edge, so a backward edge undoes earlier flow. A backward edge
is in $G(f)$ only as long as there is positive flow on its original edge. This
means that we can only undo flow that is actually there.

A negative cost cycle in $G(f)$ is a directed cycle of residual edges where the
costs sum to a negative number. The following theorem from Ahuja, Magnanti and
Orlin @ahuja1993 states when a circulation is optimal.

#theorem("Negative Cycle Optimality Theorem")[
  A feasible circulation $f^*$ is an optimal solution of the minimum cost
  circulation problem if and only if the residual network $G(f^*)$ contains no
  negative cost directed cycle.
]<negative-cycle-optimality-theorem>

At first glance the theorem may seem to say that any circulation carrying flow is
suboptimal, since pushing flow creates backward edges with positive cost. This is
not the case. If we push flow around a cycle where every edge has a cost of $-1$,
we do create backward edges of cost $+1$, but these form a cycle with positive
cost, not a negative one. So a circulation can carry flow and still have no
negative cycle in its residual network. A negative cost cycle in $G(f)$ instead
represents an improvement that is still possible. Such a cycle can use forward
edges, where we push new flow, and backward edges, where we reroute flow we
already committed. When no negative cycle is left, no improvement is left, and by
@negative-cycle-optimality-theorem $f$ is optimal.

=== Algorithm

Using the Negative Cycle Optimality Theorem we can now formalize the cycle
cancelling algorithm. The idea is simple. We start with some feasible circulation,
and while the residual network has a negative cost cycle we cancel that cycle. The
theorem tells us that when there are no negative cost cycles left, the circulation
is optimal.

The starting circulation can often be $0$ on all edges. In general, a circulation
problem can also have lower bounds on the flow of each edge, requiring some edges
to carry a minimum amount of flow. Then the zero circulation is not feasible, and a
feasible starting circulation must first be computed, which can be done with a
max-flow computation on a modified network @ahuja1993. In the circulation networks
we construct for the GP assignment problem all lower bounds will be $0$, so the
zero circulation is always feasible and we use it as the start. The outline of the
algorithm is therefore as follows:

#import "@preview/lovelace:0.3.1": *
#pseudocode-list[
  + Start with any feasible circulation $f$, in our case $f = 0$
  + *while* $exists$ negative residual cycle $c$
    + Cancel $c$: push $"capacity"(c)$ units of flow along every residual edge of $c$
]

Here $"capacity"(c)$ is the smallest residual capacity of any edge in $c$. What
pushing flow means depends on the type of residual edge. Recall that a forward edge
$(v, w, i)$ comes from an edge with leftover capacity, and a backward edge
$(w, v, i)$ comes from an edge that already has flow on it in the reverse
direction. When the cycle uses a forward edge $(v, w, i)$ we increase the flow
$f(v, w, i)$ on that edge. When the cycle uses a backward edge $(w, v, i)$ we
decrease the flow $f(v, w, i)$ on the original edge. This is how the algorithm can
undo earlier flow.

After we cancel a cycle the residual network changes. An edge that we pushed flow
on has less leftover capacity, and its forward edge can become saturated. At the
same time its backward edge gets more residual capacity, since there is now more
flow that can be undone. This is why a later cycle can push flow back using a
backward edge and undo a forward flow made earlier.

=== Runtime

Finding the negative residual cycles can be done using the Bellman-Ford algorithm
in $O(n m)$ time @ahuja1993. For each negative residual cycle that is cancelled the
total cost of the circulation decreases by at least $1$. The cost is bounded below
by $-m C U$ where $C$ is the maximum cost and $U$ is the maximum capacity of any
edge. It follows that the number of iterations is bounded by $O(m C U)$, and then
the running time is $O(n m^2 C U)$. Note that this running time is
pseudo-polynomial, as it depends largely on the size of $C$ and $U$.

Goldberg and Tarjan proved that a variant of the cycle cancelling algorithm called
Minimum Mean Cycle Cancelling has a running time that is strongly polynomial
@GoldbergCirculation. For the GP assignment problem however, we will later show
that the classical Cycle Cancelling algorithm is already polynomial.
