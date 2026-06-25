
#import "@preview/diagraph:0.3.6": *
#figure(
  render("digraph {
    node[shape=circle];
    A; B; C;
    {rank=same; B; C;}
    A -> B [label=\"p4\", color=forestgreen, fontcolor=forestgreen, penwidth=2];
    B -> C [label=\"p2\", color=forestgreen, fontcolor=forestgreen, penwidth=2];
    C -> A [label=\"p3\", color=forestgreen, fontcolor=forestgreen, penwidth=2];
    B -> A [label=\"p1\", color=\"gray80\", fontcolor=\"gray70\"];
  }"),
  caption: [Resultat: den store sykelen løser $p_4, p_3, p_2$. leksikografisk optimal.]
) <lex-result>


