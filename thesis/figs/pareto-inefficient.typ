#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    6 -> 5;
    6 -> 4;
    5 -> 4;
    5 -> 3;
    4 -> 6;
    3 -> 5;
  }"),
  caption: [An example where algorithm would chose a pareto inefficient solution. Numbers are priority values.]
)