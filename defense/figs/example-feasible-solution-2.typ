#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    node[shape=circle]
    rankdir=LR
    nodesep=0.5;
    ranksep=0.7;
    // triangle cycle, highlighted
    1 [color=forestgreen, fontcolor=forestgreen];
    2 [color=forestgreen, fontcolor=forestgreen];
    3 [color=forestgreen, fontcolor=forestgreen];
    1 -> 2 [color=forestgreen];
    2 -> 3 [color=forestgreen];
    3 -> 1 [color=forestgreen];
    // attached nodes left black
    4 -> 1;
    5 -> 2;
  }"),
  caption: [Samme graf som forrige, men løsning i grønt.]
) <feasible-example-green>
