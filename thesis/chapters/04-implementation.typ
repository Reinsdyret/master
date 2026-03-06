= Implementation <ch:implementation>


== TTC

First I started implementing the Top Trading cycles implementation described in Huitfeld's paper.
For this algorithm we are given the lists of patients and doctors $P, D$ and $D_"cur", D_"pref"$ and a priority function $R : P arrow NN$, mapping some positive integer to each patient, the higher the number the higher the priority. 
We create our graph 

Let $I = {0, ..., |P|-1}$. Then:

$G = (V, E), quad V = P union D, quad E = {(p_i, D_"pref"[i]) | i in I} union {(D_"cur"[i], p_i) | i in I}$

So we use the patients and doctors as nodes and create an edge from a patient to its preferred doctor and a doctor to all patient with that doctor as its current doctor.

Then the algorithm goes as follows:

#import "@preview/lovelace:0.3.1": *

#pseudocode-list[
  + let resolved_patients = []

  + let p_prio = Sorted list of patients by priority
  + let wants_to_switch = $["true"] * |P|$
  + *for each* $p in "p_prio"$ *do*
    + *if* let cycle = dfs_find_cycle(G, R, p) *do*
      + *for each* $p in "cycle"$ *do*
        + let $i = "idx"(p)$
        + $"wants_to_switch"[i] = "false"$
        + $"resolved_patients"."push"(p)$
      + *end*
    + *end*
  + *end*

  + *return* resolved_patients
]

The function "dfs_find_cycle" starts a dfs from patient $p$ and tries to find a cycle containing $p$. When the dfs enters a doctor node $d$ and it can choose between multiple patients to go to, it always starts by exploring the patient $p$ with highest priority $R(p)$.

While we can argue that this approach at least cares about the priority of a patient, we cannot guarantee that it always chooses the "best" solution. In that the greedy dfs might ruin for cycles that we could get without breaking any of the other cycles.

Consider this example graph:

#include "../figs/pareto-inefficient.typ"

== Exact algorithm for maximizing total switches



== Metaheuristics


