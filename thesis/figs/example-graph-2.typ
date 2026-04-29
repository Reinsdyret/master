#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    node[shape=circle]
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    5 -> 2;
    2 -> 5;
    1 -> 3;
    3 -> 4;
    4 -> 1;
  }"),
  caption: [An example graph G.]
) <example-graph-2>