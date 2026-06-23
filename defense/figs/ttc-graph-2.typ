#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    node[shape=circle]
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    A -> A;
  }"),
  caption: [Eksempel _TTC_ graf hvor $"Top"(A) = A$.]
) <ttc-graph-2>
