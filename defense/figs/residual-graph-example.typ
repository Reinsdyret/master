#import "@preview/diagraph:0.3.6": *

// Original network: edge a -> b with flow 1 of capacity 1
#figure(
  render("digraph {
    rankdir=LR;
    node[shape=circle];
    nodesep=0.8;
    a -> b [label=\"c=1, u=1\"];
  }"),
  caption: [Kant $a -> b$ har kostnad $c = 1$ og kapasitet $u = 1$.]
) <flow-before>

// Residual network G(f):
//  - forward edge a -> b is saturated (residual capacity 0), so it disappears
//  - backward edge b -> a appears with residual capacity 1
#figure(
  render("digraph {
    rankdir=LR;
    node[shape=circle];
    nodesep=0.8;
    a -> b [label=\"u=0\"]
    b -> a [label=\"c=-1, u=1\"];
  }"),
  caption: [Residual grafen $G(f)$ etter at $f(a,b) = 1$, fremover kanten er mettet og bakover kanten har kapasitet 1.]
) <flow-after>


