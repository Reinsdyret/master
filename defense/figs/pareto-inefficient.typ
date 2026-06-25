#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    rankdir=LR;
    node[shape=circle]
    nodesep=0.4; // horizontal spacing
    ranksep=0.4; // vertical spacing


    // Default node style
    node [style=filled];
    // Blue p-nodes
    p4 [fillcolor=\"#4a90d9\"];
    p2 [fillcolor=\"#4a90d9\"];
    p3 [fillcolor=\"#4a90d9\"];
    p1 [fillcolor=\"#4a90d9\"];

    // Red d-nodes
    d1 [fillcolor=\"#e05252\"];
    d2 [fillcolor=\"#e05252\"];
    d3 [fillcolor=\"#e05252\"];

    p4 -> d2;
    p2 -> d1;
    p3 -> d1;
    p1 -> d3;
    d2 -> p2;
    d2 -> p1;
    d1 -> p4;
    d3 -> p3;
    
  }"),
  caption: [Eksempel graf $G$ hvor _Greedy DFS_ ville valgt en suboptimal løsning. p$x$ betyr pasient $x$ har prioritet $x$.]
) <pareto-inefficient-graph>
