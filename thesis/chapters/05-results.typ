= Experiments and results <ch:results>
In this section we define the simulations we designed, algorithms to run on these simulations and metrics used to compare them. 
Then we present our hypotheses we had before running the experiments.
Finally we present and discuss results from the simulations.

== Experimental setup
Because of privacy and time constraints we could not use real data from Helfo.
We then proceeded to use randomly generated data, while trying to keep relative sizes compared to real numbers in Norway.
In the following sections we define how the data generation is done, our simulation model, exactly what algorithms we compared and the metrics we use to measure them.

=== Data generation <sec:data-generation>
To generate the starting state for our simulation, we start by defining our parameters.
- The number of patients
- The number of doctors
- The percentage of patients starting on a waitlist
- The number of patients entering the waitlist each day
- The number of districts, districts model geography such that doctors and patients have a district they are bound to. Patients may change district when they get a doctor in another district.
- The percentage chance that a patient requests a doctor outside of their district.
- The number of days in the simulation
- The random seed

We start by creating the districts, assigning weights at random between 0.5 and 1.5 to each district.
Then we assign doctors to each district depending on its weight, the higher the weight the more doctors a district gets, making sure each district has at least one doctor.
After we assign doctors weights at random between 0.5 and 1.5.
Based on these weights we distribute patients among doctors, the higher a doctors weight the more patients that doctor gets.
We make sure each doctor gets at least one patient.

Lastly we pick the patients that start on a waitlist.
For each patient picked we must choose a new preferred doctor.
If the patient's district contains only one doctor then the patient chooses a preferred doctor randomly out of all the doctors except their own. 
Otherwise the patient chooses a random doctor withing their own district or outside depending on the parameter.

The parameters are chosen to match the proportions of the real Norwegian GP system.
We simulate 1,000,000 patients and 1020 GPs, giving about 980 patients per GP, matching the roughly 5.6 million people and 5720 GPs in Norway @helfo2024.
The initial waitlist is 5% of the patients, matching the 297,000 people waiting to switch GP in Norway as of December 2025 @helfo2024.
We use 60 districts, giving 17 GPs per district, matching the average of about 16 GPs per municipality in Norway.
The probability that a request crosses districts is set to 11%.
As of December 2025, 33,500 of the roughly 297,000 people on a waitlist were waiting for a GP in another municipality or county than their own @helfo2024.
Note that this is the share of the outstanding waitlist and not the share of new requests, if cross district requests take longer to resolve they accumulate on the waitlist, so the true share of new requests is likely somewhat lower.
We still consider this the best available estimate.
We do not simulate the full population size as the exact algorithms based on Bellman-Ford scale with the number of GPs, running all algorithms for the full simulation length at full scale is not computationally feasible.
The proportions are what drive the dynamics of the waitlists, so we keep these matched to the real system instead.

=== Simulation model <sec:simulation-model>
The simulation runs day by day.
Each day consists of three steps.
First, every patient on the waitlist has their priority and waiting time increased by one day.
Then the algorithm being tested is run on the current state, every patient it resolves is moved to their preferred GP and removed from the waitlist.
Finally, new switch requests are added, each from a patient chosen at random among those not already waiting, and with the preferred GP chosen using the district structure described above.

Each day 180 patients submit a new switch request.
In 2025 there were about 370,000 self-chosen GP switches in Norway, of these about 218,000 went through a waitlist while the rest switched directly to GPs with open slots @helfo2024.
Our model has no free capacity, so we treat all voluntary switch demand as going through the exchange mechanism.
Scaled to our population size this gives about 180 requests per day.

We record statistics each day, the size of the waitlist before and after the algorithm runs, the number of patients resolved, the waiting times of the resolved patients, and the wall-clock time of the algorithm itself.
Waiting times are also recorded for the patients still on the waitlist when the simulation ends, so long waits are not hidden by never being resolved.

=== Algorithms compared <sec:algs-compared>
We compare five algorithms:
- _Huitfeldt TTC_, the existing mechanism we use as a baseline, described in @ch:implementation.
- _Greedy DFS_.
- _Cycle Cancelling for cardinality_.
- _Cycle Cancelling for utility_, with the linear priority function $R(p) = a$ where $a$ is days patient $p$ has waited.
- _Cycle Cancelling for strict priority_.
Each of these we ran in a 10 year simulation, long enough for the waitlists to stabilize and to see the long term behaviour of each algorithm.

In addition we run five variations of _Cycle Cancelling for utility_ with an exponential priority function $R(a) = k^(floor(a "/" 10))$, using the bases $k = 1.01, 1.05, 1.1, 1.5$ and $1.9$.
The days are divided into buckets of 10 to keep the weights within bounds, see @ch:implementation.
This means patients within the same 10 day bucket are weighted equally, the exponential variants order groups of patients rather than individuals.
Recall from @ch:implementation that the exponential priorities have a practical limit on how long a patient can wait before the weights overflow, for $k = 1.9$ this limit is 1010 days.
We therefore run the exponential variants in a smaller simulation of two years, since no patient can have waited longer than the simulation itself this keeps every variant below the limit.
The two year simulation also includes all five algorithms from the long run, so the exponential variants can be compared against them directly.

=== Metrics
To compare the algorithms we use different metrics to view the positive and negative sides of each one.

The most important metric is the size of the waitlist over time.
This is what the system as a whole cares about, a good algorithm should keep the waitlist small and stable.
Closely related is the number of patients resolved each day.

The waitlist size alone can however hide unfairness.
An algorithm can keep the waitlist small while letting a few unlucky patients wait forever.
We therefore also measure the waiting times of the resolved patients, the average, the 99th percentile and the maximum.
The 99th percentile shows how the algorithm treats its least lucky patients without being dominated by a single outlier as the maximum is.
Waiting times are also recorded for the patients still on the waitlist when the simulation ends, so long waits are not hidden by never being resolved.

We also measure the share of the waitlist that is waiting for a GP in another district.
Cross district requests should be harder to resolve as fewer patients want to switch the other way, this metric shows how each algorithm treats these patients.
Recall from @sec:data-generation that 11% of new requests are cross district, if an algorithm treats them no worse than other requests their share of the waitlist should stay near 11%.

Finally we measure the wall-clock time each algorithm uses per day.
An exact algorithm is of little practical use if it cannot keep up with the system it is meant to run in.

== Results

In this section we present the results of running simulations with the algorithms mentioned in @sec:algs-compared.
We ran three simulations in total.

The first is our main simulation.
It runs _Greedy DFS_, _Huitfeldt TTC_, _Cycle Cancelling for cardinality_, _Cycle Cancelling for strict priority_ and _Cycle Cancelling for utility_ with a linear priority function $R(p) = a$, where $a$ is the number of days patient $p$ has waited.
It runs for a simulated 10 years, or 3650 days, with the parameters defined in @sec:data-generation and @sec:simulation-model.
This is the simulation we use for most of our results.

The second simulation adds the exponential variants of _Cycle Cancelling for utility_, with bases $k = 1.01, 1.05, 1.1, 1.5$ and $1.9$, alongside the five algorithms from the main simulation.
With a base greater than one the priority function grows rapidly as a patient waits, so the weights would overflow on a long simulation.
For $k = 1.9$ this happens after 1010 days, see @ch:implementation.
We therefore run this simulation for only 730 days, short enough that no weight overflows.
It lets us compare the exponential variants against the other algorithms directly.

The third simulation is used to study the long term behaviour of the waitlists.
The main simulation runs for 10 years, which as we will see is not long enough for the waitlists to stabilize.
Running the full size system for much longer is too computationally expensive, so we instead run a smaller system, with one tenth the patients and GPs but the same proportions, for a simulated 100 years.
Since the proportions are kept the same, this smaller system models the same dynamics at a smaller scale.

=== Waitlist size over time
==== Large simulation 10 years
#include "../figs/simulation_waitlist_1million_3650_days.typ"

==== Small simulation with exponential variants
#include "../figs/simulation_waitlist_small_730_days.typ"

==== Small simulation over 100 years
#include "../figs/simulation_waitlist_small_100y.typ"

=== Maximum waiting time 
==== Large simulation over 10 years
#include "../figs/summary_large_resolved_p99_overall_max_wait.typ"

==== Small simulation with exponential variants
#include "../figs/summary_small_730_days_resolved_p99_overall_max_wait.typ"

=== Average waiting time 

=== Runtime 

== Discussion
