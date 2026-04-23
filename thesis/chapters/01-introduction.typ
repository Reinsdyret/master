= Introduction <ch:introduction>

The Norwegian general practitioner (GP) system, started in 2001, has aim to serve the Norwegian population with a stable GP.
A GP can follow his or hers patients over time, creating a trustful and safe relationship @legelisten2024.
Today this system works as intended, and GPs handle most of the populations needs.
As of December 2025 the number of GPs in Norway is 5720 and the total number of citizents in the Norwegian GP system is 5.6 million. The amount of patients each GP has varies but on average each GP has about a thousand patients @helfo2024.

/*
#figure(
  image("../figs/max_number_patients_per_GP_desember_2025.png"),
  caption: [Maximum number of patients per GP as of December 2025 @helfo2024],
) <fig:max-patients-per-gp>
*/

One inefficiency in the GP system is when a person wants to switch from one GP to another.
If the GP they want to switch to does not have any more capacity for new patients, the person has to sign up in a queue and wait for a free slot to open up with that GP. In December the number of open GP lists was 1340, indicating that 77% of all GPs had no extra capacity @ssb12005.
In a more densely populated city such as Bergen, the numbers are even worse. In December 2025 Bergen had 268 GPs and only 7 GPs that had available capacity for new patients. Thats 97% of the GPs in Bergen had no extra capacity @ssb12005.

When a person wants to switch to a GP that has no excess capacity they must sign up and then wait in line. In December 2025 there were about 300,000 people waiting for GPs in Norway @helfo2024.
As there are only 5720 doctors each doctor has on average more than 50 people on his or hers waiting list. Since many are switching from a GP it follows that it is likely that there are cycles, in that a person $P_A$ wants doctor $D_A$ which currently has $P_B$ as a patient, but $P_B$ wants to switch to doctor $D_B$ who currently has $P_A$ as a patient.
In this case patient $P_A$ ends up waiting for an open slot with $P_B$'s doctor and vice versa, but logically instead of waiting $P_A$ and $P_B$ could just swap doctors. This can be generalized to longer "waiting cycles" that could all be serviced by allowing for simultaneous switching.

In this thesis, we study this problem and come up with solutions for how one can help people who are waiting in line so that they can get their preferred choice.
In particular we propose that we can use cycle detecting algorithms, to find optimal switches between patients and massively reduce the number of people on waiting lists by using these algorithms periodically.
Our algorithms are based on existing cycle cancelling and TTC algorithms, and focus on two measures of fairness:
 - Switching as many patients as possible
 - Switching as high priority patients as possible
We have developed and tested algorithms for these cases. 

In addition we compare these optimal algorithms with the existing TTC algorithm and a greedy algorithm, comparing gain of good vs runtime.

== Thesis Outline

The remainder of this thesis is organized as follows:

- *@ch:background* provides background on the Top Trading Cycles mechanism, cycle cancelling algorithms and how these methods have been used in similar problems.
- *@ch:problem* formally defines the computational problems arising from TTC as variants of cycle cover problems on directed graphs.
- *@ch:implementation* describes our Rust implementation and the algorithmic strategies employed.
- *@ch:results* presents experimental results comparing different priority strategies.
- *@ch:conclusion* concludes and discusses future work.

