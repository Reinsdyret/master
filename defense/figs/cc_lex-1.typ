#import "@preview/diagraph:0.3.6": *

#figure(
  render("digraph {
    node[shape=circle];
    A; B; C;
    {rank=same; B; C;}
    A -> B [label=\"p4\"];
    B -> A [label=\"p1\"];
    B -> C [label=\"p2\"];
    C -> A [label=\"p3\"];
  }"),
  caption: [Noder $p_x$ har prioritet $x$.]
) <lex-instance>


