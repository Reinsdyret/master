#import "@preview/diagraph:0.3.6": *

#figure(
  render("digraph {
    node[shape=circle];
    rankdir=LR;
    {rank=same; b; c;}
    {rank=min; a;}
    a -> b [label=\"-1, 0/1\"];
    b -> d [color=red, fontcolor=red, label=\"-1, 0/1\"];
    d -> c [color=red, fontcolor=red, label=\"-1, 0/1\"];
    c -> b [color=red, fontcolor=red, label=\"-1, 0/1 \"];
    c -> a [label=\"-1, 0/1\"];
  }"),
  caption: [Start: $f = 0$. Ved $f = 0$ er residualgrafen lik originalen. Negativ sykel $d -> c -> b -> d$ har kostnad $-3$. Merking: $c, f \/ u$.]
) <cc-step1>
