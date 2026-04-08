#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    5 -> 4;
    4 -> 5;
    1 -> 2;
    2 -> 3;
    3 -> 1;
  }"),
  caption: [An example graph G.]
) <example-graph>