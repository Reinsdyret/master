= Background <ch:background>

Problems like the GP allocation problem have been studied for quite some time with a lot of variations.
In addition there are different perspectives on how to solve these problems.
While in economics they focus on the fairness of an assignment, trying to make a stable and strategy proof assignment.
In this chapter we describe similar problems for the GP allocation problem, describe the Top Trading Cycles algorithm (TTC) and Huitfeldt et.al implementation of TTC for the GP allocation problem.
Finally we describe cycle cancelling, an optimization technique that our exact algorithms are based on.

== Related problems
There are variations of assignment problems.
Usually in assignment problems we mention agents and objects and agents want objects, in the GP allocation problem the patients would be agents and doctors would be objects.
The typical assignment problems are One-to-One assignment problems, meaning one agent gets one object and an object can only be "owned" by one agent.
In addition we also have Many-to-One assignment problems, like the GP allocation problem, one agent has one object but one object can be "owned" by more than one agent.
It might be misleading to say that our patients "own" doctors but this makes more sense in other problems that we will describe, owning a doctor in our case means having that doctor as a current doctor.

=== One-to-One assignment problems
==== Housing market
The Housing Market model introduced in Shapley and Scarf @SHAPLEY197423 describes a model with traders (agents) and indivisible goods (objects).
These goods can be houses, making it a housing market. Each agent already has one house but may want to switch.
Every agent has a preference list ordering every house in order of preference, Shapley and Scarf combine all these preference lists to make a preference matrix @SHAPLEY197423.

It is in this article from Shapley and Scarf @SHAPLEY197423 that they also introduce the Top Trading Cycles algorithm and describe how it is fitting for models like the Housing Market.
We also see similarities to the GP allocation problem in that we have agents already owning an object but wanting to switch.
Two differences is the One-to-One since a doctor can have multiple patients compared to that one house can only be owned by one agent, and that in the Housing market each agent has a complete preference list while in the GP assignment problem each agent has only one preferred doctor.

==== Kidney exchange
The Kidney Exchange problem describes an issue when trying to find compatible kidney donors.
It describes the case where we have pairs of patients and donors, but the patient is incompatible with the donors kidney. 
We then need to find some way to swap around the donors to compatible patients. Either with pairwise swapping or with larger cycles.
If we visualize the problem as a directed graph, each vertex is a patient and donor pair. An edge from Pair A to Pair B means that Donor A is compatible with Patient B.
Now cycles can form as in @kidney-exchange-example we have a cycle of length 3 $A arrow B arrow C$.

#include "../figs/kidney-exchange-example.typ"

Abraham, Blum and Sandholm @ABRAHAM2007 show that, with unbounded cycle length, a maximum-cardinality and maximum-weight exchange can be found in polynomial time @enwiki:1351916222.
An issue often arising with unrestricted cycle length is if a donor suddenly refuses to donate, if this donor refuses after their patient has received a kidney then a patient is left without a donor and cannot exchange later.
Because of this the problem is often considered with a restricted cycle length to allow all operations to be executed at the same time. This way no donor can refuse in the middle.
For each pair in a cycle you need 2 operations, for a cycle of length $k$ you need $2k$ operations at the same time. 
Finding a maximum-cardinality exchange with cycles of length at most $k$, for any fixed $k >= 3$, is an NP-hard computational problem @ABRAHAM2007 @enwiki:1351916222.

The Kidney Exchange problem and the GP allocation problem have much in common.
In both problems agents already have an object, but want to switch for another.
We need to find cycles to exchange the objects in such a manner that most amount of people are happy.
The key difference is that only one donor can be "owned" by one patient while in the GP allocation problem a doctor can be "owned" by multiple patients.

=== Many-to-One assignment problems
==== The College Admissions problem 
In the College Admissions problem, introduced by Gale and Shapley, describes the problem of assigning applicants among colleges.
Each college has a certain quote, a maximum number of applicants to admit and each applicant ranks the colleges in order of preference.
An applicant can skip colleges he or she would never accept under any circumstances @GALES1962.

The problem is then to find a stable assignment meaning that there does not exist a case where there are two applicants $alpha$ and $beta$ who are assigned to colleges $A$ and $B$, respectively, although $beta$ prefers $A$ to $B$ and $A$ prefers $beta$ to $alpha$.
Gale and Shapley introduce an algorithm called deferred-acceptance (DA) to find the optimal stable assignment and prove that this runs in polynomial time and finds the optimal stable assignment @GALES1962.

There are close similarities between the College Admissions problem and the GP assignment problem.
One difference is that patients, equivalent to the applicants, only prefer one doctor.
Another difference is that patients already have existing doctors, making the problem different since applicants do not already have a college they want to switch from.

== Top Trading Cycles

Top Trading Cycles (TTC), developed by Gale and published by Shapley and Scarf @SHAPLEY197423, is an algorithm to find a re-allocation of goods without using money.
As mentioned it was introduced in an article together with the Housing Market problem.
The algorithm finds a re-allocation of houses to traders, such that all mutually-beneficial exchanges have been realized @enwiki:1344910705.
For the Housing Market problem the TTC algorithm does as follows @enwiki:1344910705:
+ Ask each agent to indicate his "top" (most preferred) house.
+ Draw an arrow from each agent $i$ to the agent, denoted $"Top"(i)$, who holds the top house of $i$.
+ Find a cycle (guaranteed to exist) and execute the trades in that cycle. Remove all involved agents from the graph.
+ If there are remaining agents, go back to step 1.

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

Roth proved that TTC is strategy proof, meaning all agents should prefer their true preferences @ROTH1982127.
TTC has also satisfies properties of Individual rationality, that an agent is at least as well of by participating, and Pareto efficiency which is that the objects are allocated such that no agent can improve without worsening another agent.

Now using TTC for the GP assignment problem is challenging as patients do not have complete strict preference lists, they only have one preferred doctor, and a doctor can be "owned" by multiple patients.
This makes the $"Top"(i)$ give out multiple patients, as the doctor that patient $i$ prefers is "owned" by multiple patients. How should we choose between these?
This is what Huitfeldt et. al has written about and how their implementation satisfies the same properties of the classical TTC. 
We try to 
== Cycle cancelling

== Related Work


