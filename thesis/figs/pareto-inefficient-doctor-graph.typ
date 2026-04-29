#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    rankdir=LR;
    node[shape=circle]
    nodesep=0.4; // horizontal spacing
    ranksep=0.8; // vertical spacing


    // Default node style
    node [style=filled];

    // Red d-nodes
    d1 [fillcolor=\"#e05252\"];
    d2 [fillcolor=\"#e05252\"];
    d3 [fillcolor=\"#e05252\"];

    d2 -> d3 [label=1];
    d2 -> d1 [label=1];
    d1 -> d2 [label=1];
    d3 -> d1 [label=1];
    
  }"),
  caption: [ Graph from @pareto-inefficient-graph but in doctor graph structure.]
) <pareto-inefficient-doctor-graph>