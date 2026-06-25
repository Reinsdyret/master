#import "@preview/diagraph:0.3.6": *

#figure(
  render("digraph {
    node[shape=circle];
    rankdir=LR;

    {rank=same; b; c;}
    {rank=min; a;}
    // live forward edges (residual capacity 1), cost -1
    a -> b [color=red, fontcolor=red, label=\"-1\"];
    c -> a [color=red, fontcolor=red, label=\"-1\"];

    // saturated forward edges kept as faint ghosts (no arrow, no flow)
    b -> d [color=\"gray85\"];
    d -> c [color=\"gray85\"];
    c -> b [color=\"gray85\"];

    // backward edges (dashed), cost +1
    d -> b [style=dashed, label=\"+1\"];
    c -> d [style=dashed, label=\"+1\"];
    b -> c [style=dashed, color=red, fontcolor=red, label=\"+1\"];
  }"),
  caption: [Residualgrafen $G(f)$: mettede fremoverkanter vises som grå, baklengskanter (stiplet) har kostnad $+1$. Negativ sykel $a -> b -> c -> a$ har kostnad $-1$.]
) <cc-step3>
