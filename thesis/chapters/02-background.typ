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
Every agent has a preference list ordering every house in order of p3reference, Shapley and Scarf combine all these preference lists to make a preference matrix @SHAPLEY197423.

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
The problem is often considered with a restricted cycle length to allow all operations to be executed at the same time. This way no donor can refuse in the middle.
For each pair in a cycle you need 2 operations, for a cycle of length $k$ you need $2k$ operations at the same time. 
Finding a maximum-cardinality exchange with cycles of length at most $k$, for any fixed $k >= 3$, is an NP-hard computational problem @ABRAHAM2007 @enwiki:1351916222.


== Top Trading Cycles

== Cycle cancelling

== Related Work


