#import "@preview/diagraph:0.3.6": *
#figure(
  render("digraph {
    node[shape=circle];
    A; B; C;
    {rank=same; B; C;}
    A -> B [label=\"p4\", color=red, fontcolor=red, penwidth=2];
    B -> A [label=\"p1\", color=red, fontcolor=red, penwidth=2];
    B -> C [label=\"p2\"];
    C -> A [label=\"p3\"];
  }"),
  caption: [Steg 1: behandle $p_4$. Vi finne sykel $A -> B -> A$ via $p_1$. $p_4$ legges til i løsningen, kanten slettes.\
$S = {"p4"}$]
) <lex-step1>


