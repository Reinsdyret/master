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
We could not run the full population size, as the exact algorithms based on Bellman-Ford scale with the number of GPs, and running all algorithms for the full simulation length at full scale was not feasible in the time we had.
Therefore we scale the system down and keep the proportions matched to the real system instead.
We simulate 100,000 patients and 102 GPs, giving about 980 patients per GP, matching the roughly 5.6 million people and 5720 GPs in Norway @helfo2024.
The initial waitlist is 5% of the patients, matching the 297,000 people waiting to switch GP in Norway as of December 2025 @helfo2024.
We use 6 districts, giving 17 GPs per district, matching the average of about 16 GPs per municipality in Norway.
The probability that a request crosses districts is set to 11%.
As of December 2025, 33,500 of the roughly 297,000 people on a waitlist were waiting for a GP in another municipality or county than their own @helfo2024.
Note that this is the share of the outstanding waitlist and not the share of new requests, if cross district requests take longer to resolve they accumulate on the waitlist, so the true share of new requests is likely somewhat lower.
We still consider this the best available estimate.
The proportions are what drive the dynamics of the waitlists, so we keep these matched to the real system instead.

=== Simulation model <sec:simulation-model>
The simulation runs day by day.
Each day consists of three steps.
First, every patient on the waitlist has their priority and waiting time increased by one day.
Then the algorithm being tested is run on the current state, every patient it resolves is moved to their preferred GP and removed from the waitlist.
Finally, new switch requests are added, each from a patient chosen at random among those not already waiting, and with the preferred GP chosen using the district structure described above.

Each day 18 patients submit a new switch request.
In 2025 there were about 370,000 self-chosen GP switches in Norway, of these about 218,000 went through a waitlist while the rest switched directly to GPs with open slots @helfo2024.
Our model has no free capacity, so we treat all voluntary switch demand as going through the exchange mechanism.
Scaled to our population size this gives about 18 requests per day.

We record statistics each day, the size of the waitlist before and after the algorithm runs, the number of patients resolved, the waiting times of the resolved patients, and the wall-clock time of the algorithm itself.
Waiting times are also recorded for the patients still on the waitlist when the simulation ends, so long waits are not hidden by never being resolved.

=== Algorithms compared <sec:algs-compared>
We compare five algorithms:
- _Huitfeldt TTC_, the existing mechanism we use as a baseline, described in @ch:implementation.
- _Greedy DFS_.
- _Cycle Cancelling for cardinality_.
- _Cycle Cancelling for utility_, with the linear priority function $R(p) = a$ where $a$ is days patient $p$ has waited.
- _Cycle Cancelling for strict priority_.
Each of these we ran in a 100 year simulation, long enough for the waitlists to stabilize and to see the long term behaviour of each algorithm.

In addition we run five variations of _Cycle Cancelling for utility_ with an exponential priority function $R(a) = k^(floor(a "/" 10))$, using the bases $k = 1.01, 1.05, 1.1, 1.5$ and $1.9$.
The days are divided into buckets of 10 to keep the weights within bounds, see @ch:implementation.
This means patients within the same 10 day bucket are weighted equally, the exponential variants order groups of patients rather than individuals.
Recall from @ch:implementation that the exponential priorities have a practical limit on how long a patient can wait before the weights overflow, for $k = 1.9$ this limit is 1010 days.
We therefore run the exponential variants in a smaller simulation of two years, since no patient can have waited longer than the simulation itself this keeps every variant below the limit.
The two year simulation also includes all five algorithms from the long run, so the exponential variants can be compared against them directly.

=== Metrics
To compare the algorithms we use different metrics to view the positive and negative sides of each one.

The first simple metric is the sum of patients resolved over the whole simulation.
This metric on its own says little about how good an algorithm is, since an algorithm that resolves many patients can still leave some waiting a long time, but it lets us see whether the algorithms differ in how many patients they resolve at all, or only in which patients they resolve.

The most important metric is the size of the waitlist over time.
This is what the system as a whole cares about, a good algorithm should keep the waitlist small and stable.

The waitlist size alone can however hide unfairness.
An algorithm can keep the waitlist small while letting a few unlucky patients wait forever.
We therefore also measure the waiting times of the resolved patients, the average, the 99th percentile and the maximum.
The 99th percentile shows how the algorithm treats its least lucky patients without being dominated by a single outlier as the maximum is.
Waiting times are also recorded for the patients still on the waitlist when the simulation ends, so long waits are not hidden by never being resolved.

We also measure the share of the waitlist that is waiting for a GP in another district.
Cross district requests shoul be harder to resolve as fewer patients want to switch the other way, this metric shows how each algorithm treats these patients.
Recall from @sec:data-generation that 11% of new requests are cross district, if an algorithm treats them no worse than other requests their share of the waitlist should stay near 11%.

Finally we measure the wall-clock time each algorithm uses per day.
An exact algorithm is of little practical use if it cannot keep up with the system it is meant to run in.

== Results

In this section we present the results of running simulations with the algorithms mentioned in @sec:algs-compared.
We ran two simulations in total.

The first is our main simulation.
It runs _Greedy DFS_, _Huitfeldt TTC_, _Cycle Cancelling for cardinality_, _Cycle Cancelling for strict priority_ and _Cycle Cancelling for utility_ with a linear priority function $R(p) = a$, where $a$ is the number of days patient $p$ has waited.
It runs for a simulated 100 years, or 36500 days, with the parameters defined in @sec:data-generation and @sec:simulation-model.

The second simulation adds the exponential variants of _Cycle Cancelling for utility_, with bases $k = 1.01, 1.05, 1.1, 1.5$ and $1.9$, alongside the five algorithms from the main simulation.
With a base greater than one the priority function grows rapidly as a patient waits, so the weights would overflow on a long simulation.
For $k = 1.9$ this happens after 1010 days, see @ch:implementation.
We therefore run this simulation for only two years or 730 days, short enough that no weight overflows.
It lets us compare the exponential variants against the other algorithms directly.

=== Total resolved
@fig-total-resolved shows the total number of resolved patients in the simulation of 100 years. Note that patients can be counted more than once as a patient can request to switch GP more than once.
#include "../figs/summary_large_total_resolved.typ"

=== Waitlist size over time
@fig-waitlist-100y and @fig-waitlist-2y shows the size of the waitlists after the algorithm has ran each day. 
How the waitlist changes in the 100 year simulation is in @fig-waitlist-100y and the two year simulation with exponential variants is in @fig-waitlist-2y.
The lists in both simulations climb under every algorithm with _Cycle Cancelling for strict priority_ ending highest and _Cycle Cancelling for cardinality_ lowest.

#include "../figs/simulation_waitlist_100y.typ"

#include "../figs/simulation_waitlist_small_730_days.typ"

=== Maximum waiting time 
@fig-max-wait-100y and @fig-max-wait-2y shows how long patients wait before being resolved or never resolved.
On the left we have how long on average the 99th percentile of patients that get resolved have to wait. 
By 99th percentile we ignore outliers that take very long and then can see how long most of the population have to wait.
On the right we have the longest amount of time a patient waits before being resolved, incuding patients that are not resolved.
If a patient is never resolved then the metric is equal to the number of days in the simulation.
For @fig-max-wait-100y this is 36500 days and for @fig-max-wait-2y this is 730 days.
_Cycle Cancelling for strict priority_, _Cycle Cancelling for utility_ and _Greedy DFS_ are all under the max bound while the others reach the full simulation length, meaning some patients are never resolved.

#include "../figs/summary_large_resolved_p99_overall_max_wait.typ"
#include "../figs/summary_small_730_days_resolved_p99_overall_max_wait.typ"

=== Average waiting time 
@fig-avg-wait-100y and @fig-avg-wait-2y show the average wait time for resolved patients.
@fig-avg-wait-100y for the 100 year simulation and @fig-avg-wait-2y for the two year simulation.
The average wait is lowest for _Cycle Cancelling for cardinality_ and highest for _Cycle Cancelling for strict priority_, with the others in between.

#include "../figs/summary_large_resolved_avg_wait.typ"
#include "../figs/summary_small_resolved_avg_wait.typ"

=== Runtime 
@fig-avg-time-100y and @fig-avg-time-2y show how many milliseconds on average each algorithm takes per day.

#include "../figs/summary_large_avg_solve_ms_total_solve_s.typ"
#include "../figs/summary_small_730d_avg_solve_ms_total_solve_s.typ"

== Discussion

The clearest thing to say about the algorithms is that they all resolve almost the same number of patients.
@fig-total-resolved shows that over the whole simulation the most and the fewest differ by just over one percent, from 622002 resolved under _Cycle Cancelling for strict priority_ to 629497 under _Cycle Cancelling for cardinality_.
Since a patient who is resolved can submit a new switch request later and rejoin the waitlist, this is a count of switches carried out over the whole run and not of distinct people, so there is no fixed total that forces the algorithms together.
That they still end up within about one percent of each other tells us that this metric is not what separates them.
They all carry out close to the same number of switches, and the real difference is in which patients they serve.
This is also what produces the differences in waiting time that we discuss below.
An algorithm that resolves the same number of patients but always picks the ones who have waited the longest looks very different from one that picks whichever patients lead to the largest solutions.

The size of the waitlist is where this first shows up.
In @fig-waitlist-100y _Cycle Cancelling for cardinality_ ends with the smallest waitlist, which is what we expected, since it resolves the largest number of patients each day.
The result we did not expect is that _Huitfeldt TTC_ follows it so closely, ending almost level with cardinality.
We thought that the _Huitfeldt TTC_ was too restrictive, but it nearly matches the algorithm that resolves the most patients.
Below these two, _Cycle Cancelling for utility_ and _Greedy DFS_ end with larger and almost equal waitlists, and _Cycle Cancelling for strict priority_ ends with the largest of all.
That strict priority keeps the largest list is expected, it spends its cycles on the requests that have waited the longest rather than on the requests that would let it resolve the most, so it clears its backlog more slowly.

The waiting times show why the size of the waitlist alone is misleading, and they reveal the central trade-off between the algorithms.
The two algorithms with the smallest waitlists, _Huitfeldt TTC_ and _Cycle Cancelling for cardinality_, also give the shortest waits for the typical request.
In @fig-avg-wait-100y their average waits are by far the lowest, 352.9 days for _Huitfeldt TTC_ and 940.7 days for _Cycle Cancelling for cardinality_, against more than 1300 days for each of the other three.
But the same two algorithms have the longest waits for the unluckiest requests.
In the maximum wait in @fig-max-wait-100y they reach 36500 and 36172 days, essentially the entire length of the simulation, while the other three keep their maximum well below this.
So the two algorithms that serve the typical request the fastest are exactly the ones that let a few requests wait almost the whole simulation, and the algorithms that protect those few requests do so by making the typical request wait longer.
This trade-off runs through all of our results.

The 99th percentile wait in @fig-max-wait-100y shows the same split from a less extreme angle.
Here _Cycle Cancelling for cardinality_ is the worst at 14219 days and _Cycle Cancelling for strict priority_ is the best at 9335 days, with the others in between.
So even short of the absolute maximum, cardinality lets its slowest requests wait the longest, while strict priority keeps them the most contained.

_Cycle Cancelling for strict priority_ is the clearest example of the protective end of this trade-off.
It has the worst average wait, at 1384.1 days in @fig-avg-wait-100y, but the best 99th percentile wait at 9335 days and the best maximum wait at 11955 days in @fig-max-wait-100y.
This follows from the ordering.
Because a request with higher priority always counts for more than any number of requests with lower priority, and our priority grows with the days a request has waited, the algorithm serves the longest waiting requests first whenever a cycle through them exists.
So a request that keeps waiting climbs in priority until the algorithm resolves it, and no single request sits unresolved far longer than the rest, even though a patient resolved on one request may submit another later.
The price is that the algorithm spends its cycles on requests that have already waited a long time rather than on the requests that would let it resolve the most, so the typical request waits longer.

_Greedy DFS_ and _Cycle Cancelling for utility_ sit between the two ends on the maximum wait.
Neither reaches the full length of the simulation, with maximum waits of 13416 and 17743 days in @fig-max-wait-100y, so unlike _Huitfeldt TTC_ they do not let any request wait the whole simulation.
_Greedy DFS_ does this because it goes through the requests in order of priority, like _Cycle Cancelling for strict priority_, so the longest waiting ones are served whenever a cycle through them exists, but its greedy and sometimes suboptimal choices give it a worse 99th percentile and maximum wait than _Cycle Cancelling for strict priority_, at 9763 and 13416 days against 9335 and 11955 in @fig-max-wait-100y.
_Cycle Cancelling for utility_ also keeps the maximum wait bounded, since with our linear priority a request that keeps waiting gains weight until it is resolved.
This is worth noting, since one might expect the linear utility ordering to let a single long waiting request be outweighed by a group of requests with shorter waits and so be left behind, but over a long simulation the weight of a waiting request grows without bound and eventually forces its resolution.
On the typical request, though, _Cycle Cancelling for utility_ behaves more like strict priority than like cardinality, with an average wait of 1331.6 days, because over a hundred years the linearly growing weights come to favour long waiting requests strongly.

It is worth asking which requests are the ones left waiting almost the whole simulation under _Cycle Cancelling for cardinality_ and _Huitfeldt TTC_.
A request can only be resolved when the GP it prefers can reach the GP it currently holds through a chain of other patients who want to switch, that is, when the request lies on a cycle.
Two situations make this unlikely.
The first is when few or no other patients want the request's current GP, so there is no one to take over that slot and close a cycle through it.
The second is when the request prefers a GP that few patients are trying to leave, whether because its panel is small or because its patients are mostly content, so a chain can rarely continue past that GP.
In both cases the request lies on few cycles, or none, and so has little chance of being picked when cardinality resolves the requests that lead to the largest solutions.
Since the same patient can submit such a request again and again, these are requests that stay stuck across the whole run rather than a one off piece of bad luck, and they are a property of individual GPs and not of geography.
One might expect them to be requests for a GP in another district, since fewer patients want to switch in the opposite direction across a district boundary, but this is not the case, as shown in @fig-district.

Runtime is the last thing that separates the algorithms, and here the order between them changes with the size of the system.
In the main simulation in @fig-avg-time-100y _Cycle Cancelling for cardinality_ is by far the fastest, at 9.2 milliseconds per simulated day against 128 to 379 milliseconds for the others.
The reason is that it works on the collapsed graph, where the nodes are the GPs and all patients wanting the same switch share one edge, so its running time grows with the number of GPs rather than the number of patients.
The other algorithms keep each patient separate, so their running time grows with the number of patients, which is far larger, and _Cycle Cancelling for strict priority_ and _Greedy DFS_, which run a separate graph search for each patient, are the slowest at 378.8 and 374.5 milliseconds per day.
In the smaller two year simulation in @fig-avg-time-2y the order changes, _Greedy DFS_ is the fastest at 1.5 milliseconds per day and the utility variants are the slowest at around 20, since with far fewer patients the per patient searches become cheap while the heavier work the utility solver does on its larger graph does not shrink as much.

The exponential variants of _Cycle Cancelling for utility_ confirm that the utility ordering is a single spectrum with cardinality and lexicographic priority at its ends, as we argued in @ch:problem.
This is clearest in the average wait in @fig-avg-wait-2y, where the variants rise steadily with the base, from 32.5 days at base 1.01, almost the same as the 30.9 days of _Cycle Cancelling for cardinality_, up to 47.5 days at bases 1.5 and 1.9, approaching the 53.7 days of _Cycle Cancelling for strict priority_.
At a base near one the utility ordering is almost the same as maximising the number of patients resolved, which is what the theory predicts, and as the base grows it favours long waiting requests more strongly.
The waitlist size in @fig-waitlist-2y does not order the variants as cleanly.
The lowest base variant still ends with one of the smallest waitlists and the higher base variants with larger ones, but in between the variants are interleaved with each other and with the other algorithms rather than lining up by base.
We expected the waitlist size to follow the base in the same way the average wait does, and it does not.
The likely reason is that the waitlist size depends on which requests happened to be resolved early and in what order, and over only 730 days, with the lists still small and climbing, this leaves enough day to day variation to scramble variants that resolve almost the same number of requests.

There is one thing that has to be remembered when reading all of these results, which is that our simulations assume every GP is at full capacity.
As we noted in @ch:introduction not all GPs in Norway are full, and in reality slots open all the time as patients move away, pass away or as the panel caps change.
In our closed model a request can only ever be resolved by an exchange, so the requests that lie on few cycles, the same ones that give the long maximum waits above, are resolved slowly if at all and build up on the list over time.
This is why the waitlists in @fig-waitlist-100y keep climbing even after a hundred years.
In reality a patient whose request lies on few cycles would be served the next time a nearby slot opens up rather than waiting indefinitely, which is part of why the current system works and why no one waits forever in practice.
A real deployment of any of our algorithms would, like the mechanism of Huitfeldt et al. @NBERw32458, first run a waitlist step that fills the open slots and only then run the exchange.

For this reason the climbing waitlists should not be read as a prediction about Norway, they are a property of the closed model and not of the algorithms.
What the experiments do show is that the exchange resolves requests that would otherwise keep waiting, so running one of these algorithms on top of the existing waitlist clearing should make the lists shorter.
How much shorter, under the real rates at which patients arrive and leave, our model cannot say.
What our experiments do establish is the relative ordering of the algorithms, and this is what the chapter rests on.
Our results still come with limitations.
We use randomly generated data instead of real data from Helfo, although we keep the proportions matched to the real Norwegian numbers.
We have also not analysed strategic behaviour in the repeated setting, which as Huitfeldt et al. @NBERw32458 show can break the properties that hold in a single run, since our priorities depend on the days a request has waited and a patient could in principle time a request to change their priority.
Finally the requests left waiting almost the whole simulation under _Cycle Cancelling for cardinality_ and _Huitfeldt TTC_ are a real problem that a deployment would have to handle, for instance by combining the high throughput of cardinality with a rule that forces the resolution of any request that has waited beyond some limit.
