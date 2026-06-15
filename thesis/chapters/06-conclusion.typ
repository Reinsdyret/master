= Conclusion <ch:conclusion>
Lastly we try to summarize our work and point out future work and how this thesis can be extended.
== Summary

This thesis studied how the waiting lists in the Norwegian GP system can be reduced by resolving waiting cycles, cases where a group of patients each want a GP that another patient in the group currently holds.
As of December 2025 about 300000 people were waiting to switch GP in Norway @helfo2024, and because almost all GP lists are full these patients can only be served by exchanging slots among themselves.
We formalised this as the problem of finding vertex disjoint cycles in a directed graph of patients and GPs, and defined three notions of an optimal set of switches, resolving as many patients as possible, serving the highest priority patients first in lexicographic order, and a utility ordering that weighs every patient by their priority and that contains the other two as its endpoints.
We implemented an exact algorithm based on cycle cancelling for each ordering, a fast _Greedy DFS_ heuristic, and a port of the TTC mechanism of Huitfeldt et al. @NBERw32458 as a baseline, and compared them in a simulation of the Norwegian system.
The main finding is that the choice of objective has almost no effect on how many patients are resolved, the algorithms differ by only about one percent, but a large effect on which patients are resolved and therefore on how long they wait.
_Cycle Cancelling for cardinality_ keeps the smallest waitlist and the lowest typical wait, and is also the fastest of our algorithms, but lets a small number of requests wait almost indefinitely.
_Cycle Cancelling for strict priority_ makes the opposite trade, it gives the typical request the longest wait but keeps the worst case the most contained, since a request that keeps waiting climbs in priority until it is resolved.
The utility ordering sits between these two, and its base acts as a single dial between resolving the most patients and protecting those who have waited longest.
We did not expect the TTC baseline to keep pace with _Cycle Cancelling for cardinality_ on the size of the waitlist, but it does, despite its more restrictive cycle structure.

== Future Work

Several directions remain open.
The most immediate is to run the algorithms on real data from Helfo rather than the randomly generated data we use, which would test whether the proportions we assumed hold in practice and would turn our relative comparison of the algorithms into a quantitative estimate of how much they could shorten the waitlists in Norway.
Related to this, our simulations assume a closed system in which every GP is at full capacity and slots open only through exchange, which is why the waitlists keep climbing over a hundred years.
A more realistic model would let slots open as patients move away, pass away or as panel capcities change, and would run a waitlist step that fills these open slots before the exchange, as the mechanism of Huitfeldt et al. @NBERw32458 does.

A second direction concerns the requests left waiting almost the whole simulation under _Cycle Cancelling for cardinality_ and _Huitfeldt TTC_.
These are requests that lie on few cycles, and since the same patient can submit them again and again they stay stuck across the whole run.
A practical mechanism might combine the high throughput of _Cycle Cancelling for cardinality_ with a rule that forces the resolution of any request that has waited beyond a fixed limit, keeping the waitlist small while still bounding the worst case.

A third direction is strategic behaviour in the repeated setting.
Our priorities grow with the days a request has waited, so a patient could in principle time a request to gain priority, and Huitfeldt et al. @NBERw32458 show that TTC style mechanisms can lose strategy proofness and leave some patients worse off than first come first served when run repeatedly.
We have not analysed this for our algorithms, and a formal treatment is left as future work.

Finally there is room to make the exact algorithms faster and richer.
The utility algorithm is limited by its dependence on the size of the priority weights, and a strongly polynomial method such as the minimum mean cycle cancelling of Goldberg and Tarjan @GoldbergCirculation could remove this limit.
One could also restrict the cycle length, as is done in kidney exchange so that all swaps can be carried out at once, or let GPs express preferences over patients, making the problem two sided.

