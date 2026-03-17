#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    rankdir=LR;
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
  caption: [ Example graph G where algorithm would choose suboptimal solution. p$x$ means patient $x$ and has priority $x$.]
) <pareto-inefficient-graph>