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
It might be misleading to say that our patients "own" doctors but this makes more sense in other problems that we will describe, owning a doctor in our case means having that doctor as a current doctor.

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
The key difference is that only one donor can be "owned" by one patient while in the GP allocation problem a doctor can be "owned" by multiple patients.

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
As mentioned it was introduced in an article together with the Housing Market problem.
The algorithm finds a re-allocation of houses to traders, such that all mutually-beneficial exchanges have been realized @enwiki:1344910705.

TTC works on a directed graph, for the following implementation, on the Housing Market problem, each agent is a node and edges point from agents to other agents.
The algorithm then does the following @enwiki:1344910705:
+ Query each agent for its "top" (most preferred) house.
+ Insert a directed edge from each agent $i$ to the agent, denoted $"Top"(i)$, that holds the most desired house of $i$.
+ Find a cycle (which is guaranteed to exist) and execute the trades in that cycle. Next, remove all involved agents from the graph.
+ If there are remaining agents, repeat from step 1.

Note that a cycle is guaranteed to exist if there are agents still left.
This is because of the pidgeonhole principle, since each agent has one outgoing edge we have to have a cycle.
The cycle can also be of length 1 in which case an agent's top house is its own, then we treat it as a normal cycle and remove the agent from the graph.
For this classical TTC implementation to work we need to remember that agents have a strict preferences.
Each agent has a complete list of all houses ranked in order of preference.
Because of this as houses get re-allocated agents are bound to either find a trade cycle with others or end up having their $"Top"(i)$ house as their own.

Consider the example of 3 agents, $A, B "and" C$, each has a house and have a strict preference list.
Lets go through how TTC would handle this:\
The preference lists are 
- $A = [B, C, A]$
- $B = [C, A, B]$
- $C = [B, C, A]$

If we make the graph where each agent point to their top we get @ttc-graph.
#include "../figs/ttc-graph.typ"
We have the cycle $B arrow C$ so we execute the trades, remove agents from the graph and create the edges again.
The last remaining node is $A$ and as $B$ and $C$ are not in the graph, $"Top"(i)$ of $A$ is now $A$.
#include "../figs/ttc-graph-2.typ"
We execute this trade and are left with no more agents, making TTC terminate.

Roth proved that TTC is strategy proof, meaning all agents are motivated to prefer their true preferences @ROTH1982127.
TTC has also satisfies properties of Individual rationality, that an agent is at least as well of by participating, and Pareto efficiency which is that the objects are allocated such that no agent can improve without worsening another agent.

Now using TTC for the GP assignment problem is challenging as patients do not have complete strict preference lists, they only have one preferred doctor, and a doctor can be "owned" by multiple patients.
This makes the $"Top"(i)$ give out multiple patients, as the doctor that patient $i$ prefers is "owned" by multiple patients. How should we choose between these?
This is what Huitfeldt et. al has written about and how their implementation satisfies the same properties of the classical TTC. 
Later we run the TTC made for the GP assignment problem and compare with our algorithms. First we explain it and how it is different from normal TTC.

=== GP assignment problem TTC
In the paper from Huitfeldt et al. they tackle a more complex dataset as they use real patient and doctor data from Norway.
In their model they have doctors with available capacity and patients switching to these GPs can be allocated that GP without an exchange.
This is why in each of the two TTC implementations they first run a waitlist algorithm that goes through all GP with available capacity and assigns the highest priority to that GPs panels until there are no more patients waiting on GPs with available capacity @NBERw32458.
Following we explain Huitfeldt et al. TTC implementation without regard to capacities to doctors as that is how we tackle the problem.

Now the algorithm is as follows @NBERw32458:
+ Each patient points to their preferred GP, if the preferred GP is not in the graph the patient points to its current GP. Each GP points to the patient in their panel with highest priority.
+ Find a cycle in the graph, remove patients part of that cycle. If a GP has no more patients that are in the graph it is removed from the graph.
+ Repeat step 1 until there are no more patients

Like the classical TTC we are guaranteed to have a cycle because of the pidgeonhole principle, since if a patient does not get their preferred GP they end up pointing to their own GP making a cycle.

== Cycle cancelling <cycle-cancelling>
Cycle cancelling is a technique used in flow and circulation problems to find a minimum cost solution.
First we explain some of the terms needed for cycle cancelling then we go onto what cycle cancelling is.
We use definitions from Ahuja, Magnantti and Orlin @ahuja1993.

=== The minimum cost circulation problem

Let $G = (V, A)$ be a directed multigraph with $n$ vertices and $m$ edges. Each edge
is a triple $(v, w, i)$ where $v$ is the tail, $w$ is the head and $i$ is an index.
The pair $(v, w)$ gives the endpoints of the edge. The index $i$ separates edges that
have the same endpoints, so $G$ can have parallel edges. This means an edge is
identified by the full triple and not just by its endpoints.

The multigraph $G$ is a circulation network if each edge $(v, w, i)$ has a capacity
$u(v, w, i) >= 0$ and a cost $c(v, w, i) in ZZ$. For a vertex $w in V$ we let
$"in"(w) = {(v, w, i) in A}$ be the edges whose head is $w$, and
$"out"(w) = {(w, v, i) in A}$ the edges whose tail is $w$.

#definition("Circulation")[
  A circulation is a function $f$ that assigns a value $f(v, w, i)$ to each edge and
  satisfies the capacity constraints
  $
    0 <= f(v, w, i) <= u(v, w, i) quad quad forall (v, w, i) in A
  $
  and the conservation constraints
  $
    sum_((u, w, i) in "in"(w)) f(u, w, i)
      = sum_((w, v, j) in "out"(w)) f(w, v, j) quad quad forall w in V.
  $
]

The conservation constraint says that at each vertex the flow coming in equals the
flow going out. No flow enters or leaves $G$ from the outside, so all flow
circulates around the network. The cost of a circulation $f$ is
$
  "cost"(f) = sum_((v, w, i) in A) c(v, w, i) f(v, w, i).
$

#definition("Minimum cost circulation problem")[
  Given a circulation network $G$, the minimum cost circulation problem is to find
  a circulation $f$ with minimum cost.
]

In our problem each edge is a possible patient switch, and we want a circulation
that does as many switches as possible. We get this by setting the cost of every
edge to $-1$. Then $"cost"(f) = - sum_((v,w,i) in A) f(v,w,i)$, which is the negative
of the total flow. So a circulation with minimum cost is the same as a circulation
with the most flow. We do not treat the amount of flow as a separate goal. It is
already part of the cost, and the problem stays a normal minimum cost circulation
problem.

An example circulation network is shown in @example-circulation. For simplicity
this example is not a multigraph. Each edge is labeled "x,y", meaning the edge has
cost $x$ and capacity $y$, and as explained above every edge has cost $-1$. The
network has three feasible circulations: no flow on any edge, the cycle
$d arrow c arrow b arrow d$, and the cycle $a arrow b arrow d arrow c arrow a$.
Their costs are $0$, $-3$ and $-4$. The longer cycle is the optimal circulation
since it has the most flow and the lowest cost.

#include "../figs/example-circulation.typ"

=== The residual network

The residual network shows what capacity is left after a circulation, and what
flow can be undone. Given a circulation network $G$ and a circulation $f$, we build
the residual network $G(f)$ edge by edge. Each edge $(v, w, i) in A$ gives two
residual edges.

- A forward edge $(v, w, i)$ with cost $c(v, w, i)$ and residual capacity
  $u_f (v, w, i) = u(v, w, i) - f(v, w, i)$.
- A backward edge $(w, v, i)$ with cost $-c(v, w, i)$ and residual capacity
  $u_f (w, v, i) = f(v, w, i)$.

A residual edge is in $G(f)$ only if its residual capacity is greater than $0$. An
edge with residual capacity $0$ is saturated. We build the residual edges from each
edge $(v, w, i)$ and not from the pair $(v, w)$, so the index $i$ is kept on both
residual edges. This means $G(f)$ is also a directed multigraph and every residual
edge is still uniquely identified.

A forward edge has the same cost as its original edge, and pushing flow on it
increases $f$ on that edge. A backward edge has the negated cost, and pushing flow on
it decreases $f$ on the original edge. So a backward edge undoes earlier flow. A
backward edge is only in $G(f)$ up to the flow on its original edge, so we can only
undo flow when there is flow to undo.

A negative cost cycle in $G(f)$ is a directed cycle of residual edges where the
costs sum to a negative number. The following theorem from Ahuja, Magnanti and
Orlin @ahuja1993 tells us when a circulation is optimal.

#theorem("Negative Cycle Optimality Theorem")[
  A feasible circulation $f^*$ is an optimal solution of the minimum cost
  circulation problem if and only if the residual network $G(f^*)$ contains no
  negative cost directed cycle.
]<negative-cycle-optimality-theorem> 

Note that the theorem does not say every circulation with flow is suboptimal. If we
push flow around a cycle of cost $-1$ edges we do create backward edges of cost $+1$,
but these form a positive cost cycle and not a negative one. A negative cost cycle
is instead an improvement we can still make. It can use forward edges where we add
more switches, and backward edges that reroute flow we already committed. When there
is no such cycle left, there is no improvement left, and $f$ is optimal.

=== Algorithm

Using the Negative Cycle Optimality Theorem we can now formalize the cycle
cancelling algorithm. The idea is simple. We start with some feasible circulation,
and while the residual network has a negative cost cycle we cancel that cycle. The
theorem tells us that when there are no negative cost cycles left, the circulation
is optimal.

Note that the starting circulation can be $0$ on all edges. Sometimes a circulation
problem has lower bounds that make this an invalid starting circulation. For cases
with lower bounds on edges, a starting circulation can be computed using any
max-flow algorithm. In our problem all lower bounds are $0$, so the zero
circulation is always feasible and we can use it as the start.

#import "@preview/lovelace:0.3.1": *
#pseudocode-list[
  + Start with any feasible circulation $f$, this can be $0$
  + *while* $exists$ negative residual cycle $c$
    + Cancel $c$: $forall a in c: f(a) := f(a) + "capacity"(c)$
]

When we cancel a cycle we push flow equal to $"capacity"(c)$ along each residual
edge in the cycle. Here $"capacity"(c)$ is the smallest residual capacity of any edge
in $c$. Recall that the residual network has two kinds of edges. A forward edge
$(v, w, i)$ comes from an edge with leftover capacity, and a backward edge
$(w, v, i)$ comes from an edge that already has flow on it.

How we push flow depends on the type of edge. When the cycle uses a forward edge
$(v, w, i)$ we increase the flow $f(v, w, i)$ on that edge. When the cycle uses a
backward edge $(w, v, i)$ we decrease the flow $f(v, w, i)$ on the original edge.
This is how the algorithm can undo earlier flow. A backward edge is in the residual
network only up to the flow on its original edge, so we can only undo flow that is
actually there.

After we cancel a cycle the residual network changes. An edge that we pushed flow
on has less leftover capacity, and its forward edge can become saturated. At the
same time its backward edge gets more residual capacity, since there is now more
flow that can be undone. This is why a later cycle can push flow back on a backward
edge and undo a push we made before.

=== Runtime

Finding the negative residual cycles can be done with Bellman-Ford in $O(n m)$ time
@enwiki:bellman-ford. At each negative residual cycle that is cancelled the total
cost of the circulation decreases by at least $1$. The cost is bounded below by
$-m C U$ where $C$ is the max cost and $U$ is the max capacity of any edge. It
follows that the number of iterations is bounded by $O(m C U)$, and then the
runtime is $O(n m^2 C U)$. Note that this runtime is pseudo-polynomial since it
depends largely on the size of $C$ and $U$.

Goldberg and Tarjan proved that a variant of the cycle cancelling algorithm called
Minimum Mean Cycle Cancelling has a strongly polynomial bound on its runtime
@GoldbergCirculation. But we will later show how for our problem we still have a
polynomial algorithm using the classical Cycle Cancelling algorithm.
