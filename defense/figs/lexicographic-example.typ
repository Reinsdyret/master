#import "@preview/diagraph:0.3.6": *

#figure(
  render("digraph {
    node[shape=circle]
    nodesep=0.4;
    ranksep=0.3;

    // red cycle
    5 -> 4 [color=red];
    4 -> 5 [color=red];
    4 [color=red, fontcolor=red];

    // blue cycle
    5 -> 3 [color=blue];
    3 -> 2 [color=blue];
    2 -> 5 [color=blue];
    3 [color=blue, fontcolor=blue];
    2 [color=blue, fontcolor=blue];

    {rank=same; 4; 3;}
  }"),
  caption: [ En instans med to mulige løsninger\
  rød: $S = {5,4} quad chi(S) = {1,1,0,0}$\
  blå: $S' = {5, 3, 2} quad chi(S') = {1,0,1,1}$\
  $S succ_"lex" S'$]
) <lex-twocolor>
