#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    node[shape=circle]
    rankdir=LR;
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    A -> B;
    B -> C;
    C -> B;
  }"),
  caption: [An example Housing Market for TTC.]
) <ttc-graph>