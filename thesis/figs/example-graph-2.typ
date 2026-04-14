#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    rankdir=LR;
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    1 -> 2;
    2 -> 3;
    3 -> 1;
    3 -> 4;
    5 -> 4;
    4 -> 5;
    4 -> 1;
  }"),
  caption: [An example graph G.]
) <example-graph-2>