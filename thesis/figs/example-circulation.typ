#import "@preview/diagraph:0.3.6":*

#figure(
  render("digraph {
    node[shape=circle];
    edge [labeldistance=3.0];
    splines=curved;
    forcelabels=true;

    // force rank order
    a [pos=\"0,0!\"];
    b [pos=\"2,1!\"];
    c [pos=\"2,-1!\"];
    d [pos=\"4,0!\"];

    
    a -> b [headlabel=<1,1>];
    b -> a [headlabel=<1,-1>];
    c -> a [headlabel=<1,1>];
    a -> c [headlabel=<1,-1>];
    b -> d [headlabel=<1,1>];
    d -> b [headlabel=<1,-1>];
    d -> c [headlabel=<1,1>];
    c -> d [headlabel=<1,-1>];
    
  }", engine: "neato"),
  caption: [An example circulation network.]
) <example-circulation>