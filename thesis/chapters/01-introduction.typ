= Introduction <ch:introduction>

The norwegian general practitioner (GP) system, started in 2001, has aim to serve the norwegian population with a stable GP.
This GP can follow their patients over time, with focus that creates a trustful and safe relation. @legelisten2024
Today this system runs great, GPs are respected and handles most of the populations needs.
As of December 2025 the number of GPs in Norway is 5720 and the total number of citizents in the norwegian GP system is 5.6 million, so about a thousand patients per GP. @helfo2024

/*
#figure(
  image("../figs/max_number_patients_per_GP_desember_2025.png"),
  caption: [Maximum number of patients per GP as of December 2025 @helfo2024],
) <fig:max-patients-per-gp>
*/

The current inefficiency in the GP system is when a person wants to switch their GP. They then have to find a GP they would rather want and switch to it.
If that GP has a full list, the person has to get in a queue for a space at that GP. In December the number of open GP lists was 1340, so about 77% of all GPs had no extra capacity. @ssb12005
In a more densely populated city like Bergen, the numbers are even worse. In December 2025 Bergen had 268 GPs and only 7 GPs that were not full, thats 97% of GPs in Bergen had no extra capacity. @ssb12005

When a person wants to switch GP they often have to choose a full GP and then wait in line. In December 2025 there was about 300,000 people in waitlists for GPs in Norway. @helfo2024
This many in waitlists on only 5720 doctors leads to a very dense network, meaning many people want the same doctor or each others doctors. We can then often encounter cycles, in that a person $P_A$ wants doctor $D_A$ which currently has $P_B$ as a patient, but $P_B$ wants to switch to doctor $D_B$ who currently has $P_A$ as a patient.
With this case patient $P_A$ ends up waiting for an open space in $P_B$'s doctor and vice versa, but logically instead of waiting $P_A$ and $P_B$ could just swap doctors.

We propose that we can, using cycle detecting algorithms, in polynomial time find optimal switches between patients and massively reduce the number of people on waitlists by using these algorithms periodically.
Our algorithms are based on existing cycle cancelling and TTC algorithms, and focus on two measures of "good":
 - Switching as many patients as possible
 - Swtiching as high priority patients as possible
And we have developed algorithms for these cases. 

In addition we compare these optimal algorithms with the existing TTC algorithm and a greedy algorithm, comparing gain of good vs runtime.

== Thesis Outline

The remainder of this thesis is organized as follows:

- *@ch:background* provides background on the Top Trading Cycles mechanism, cycle cancelling algorithms and how these methods have been used in similar problems.
- *@ch:problem* formally defines the computational problems arising from TTC as variants of cycle cover problems on directed graphs.
- *@ch:implementation* describes our Rust implementation and the algorithmic strategies employed.
- *@ch:results* presents experimental results comparing different priority strategies.
- *@ch:conclusion* concludes and discusses future work.

