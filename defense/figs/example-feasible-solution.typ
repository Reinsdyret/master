#import "@preview/diagraph:0.3.6": *

#figure(
  render("digraph {
    node[shape=circle]
    rankdir=LR
    nodesep=0.5;
    ranksep=0.7;
    // triangle cycle
    1 -> 2;
    2 -> 3;
    3 -> 1;
    // attached but never on a cycle
    4 -> 1;   // only outgoing: nothing returns to 4
    5 -> 2;
  }"),
  caption: [En instans av _The GP assignment problem_, noder er pasienter og kant (a,b) betyr at a vil ha b sin lege.]
) <feasible-example>
