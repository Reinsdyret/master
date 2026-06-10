= Introduction <ch:introduction>
The Norwegian general practitioner (GP) system, launched in 2001, aims to ensure the Norwegian population has a stable GP.
A GP can follow their patients over time, creating a trusting and safe relationship @legelisten2024.
Today, this system works as intended, and GPs handle most of the population's needs.
As of December 2025, there are 5720 GPs in Norway serving a population of 5.6 million.
The number of patients each GP has varies, but on average each GP has about 1,000 patients @helfo2024.

/*
#figure(
  image("../figs/max_number_patients_per_GP_desember_2025.png"),
  caption: [Maximum number of patients per GP as of December 2025 @helfo2024],
) <fig:max-patients-per-gp>
*/

One inefficiency in the Norwegian GP system is when a person wants to switch from one GP to another.
If the GP they want to switch to does not have any free capacity for new patients, the person has to sign up in a queue and wait for a free slot to open up.
In December 2025, the number of open GP lists was 1340, indicating that 77% of all GPs had no extra capacity @ssb12005.
In a more densely populated city such as Bergen, the numbers were even worse.
Only 7 out of 268 GPs had available capacity for new patients.
Thus, 97% of the GPs in Bergen had no extra capacity @ssb12005.

When a person wants to switch to a GP with no excess capacity, they must sign up and wait in line.
Open slots are allocated on a first-come, first-serve basis.
In December 2025, about 300,000 people were waiting to see a GP in Norway @helfo2024.
As there were only 5720 GPs, each had an average of more than 50 people on their waiting list
Since there is a relatively large number of people who are waiting to either switch or be allocated a GP, it follows that it is likely that there are cycles, in that a person $P_A$ wants GP $D_A$, who currently has $P_B$ as a patient, but $P_B$ wants to switch to GP $D_B$, who currently has $P_A$ as a patient.
In this case, patient $P_A$ ends up waiting for an open slot with $P_B$’s GP, and vice versa. Logically, instead of waiting, $P_A$ and $P_B$ could just swap GPs.
This can be generalized to longer "waiting cycles" that could all be serviced by allowing for simultaneous switching.

In this thesis, we study how such cycles can be resolved and propose different strategies for resolving these.
In particular we investigate the use of cycle detecting algorithms to find optimal switches between patients and show that this can lead to a substantial reduction in the number of people on waiting lists by running these algorithms periodically.
Our algorithms are based on existing cycle cancelling and TTC algorithms.
While the focus is always on helping as many patients as possible we study two different variations of this problem, when there is a priority among those waiting and when there is no priority.
We have developed and tested optimal algorithms as well as heuristics for these cases and evaluated them for solution quality as well as speed.

In addition we compare our result with those of the existing algorithms.
We note that our solutions have applications to other types of reallocation problems where users want to switch their given assignment.
This could for instance be in reallocation of houses, kidney donors or students switching schools.

== Thesis Outline

The remainder of this thesis is organized as follows:

- *@ch:background* provides background on the TTC mechanism, cycle cancelling algorithms and how these methods have been used on similar problems.
- *@ch:problem* formally defines the computational problems arising from TTC as variants of cycle cover problems on directed graphs.
- *@ch:implementation* describes the algorithmic strategies employed in our implementations.
- *@ch:results* presents experimental results comparing different priority strategies.
- *@ch:conclusion* concludes and discusses future work.

