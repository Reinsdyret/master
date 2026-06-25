#import "@preview/diagraph:0.3.6": *
#figure(
  render("digraph {
    node[shape=circle];
    rankdir=LR;
    {rank=same; b; c;};
    {rank=min; a;}
    a -> b [label=\"-1, 1/1\" color=green];
    b -> d [label=\"-1, 1/1\" color=green];
    d -> c [label=\"-1, 1/1\" color=green];
    c -> b [label=\"-1, 0/1\" ];
    c -> a [label=\"-1, 1/1\" color=green];
  }"),
  caption: [Etter andre kansellering: kostnad $-4$, sirkulasjonen $a -> b -> d -> c -> a$ — optimal.]
) <cc-step4>
