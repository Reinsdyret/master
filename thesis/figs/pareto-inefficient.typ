#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    nodesep=0.6; // horizontal spacing
    ranksep=0.8; // vertical spacing
    4 -> 3;
    4 -> 2;
    3 -> 4;
    2 -> 1;
    1 -> 4;
  }"),
  caption: [ DANGER!!!! PERSON WITH PRIO 6 HAS TWO PREFERENCES HERE! An example where algorithm would chose a pareto inefficient solution. Numbers are priority values.]
)