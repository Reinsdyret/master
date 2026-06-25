
#import "@preview/diagraph:0.3.6": *
#figure(
  render("digraph {
    node[shape=circle];
    A; B; C;
    {rank=same; B; C;}
    B -> A [label=\"p1\", color=\"gray80\", fontcolor=\"gray70\"];
    A -> B [label=\"angre p1\", style=dashed, color=red, fontcolor=red];
    B -> C [label=\"p2\", color=red, fontcolor=red, penwidth=2];
    C -> A [label=\"p3\", color=red, fontcolor=red, penwidth=2];
  }"),
  caption: [Steg 2: behandle $p_3$. $p_4$ er allerede bundet (kant fjernet). Eneste vei $A → C$ bruker baklengskanten $A -> B$ som angrer $p_1$. Sykelen $C -> A -> B -> C$ legger $p_3$ i løsningen og øker flyt på $p_2$.\
$S = {"p4", "p3"}$]
) <lex-step2>

