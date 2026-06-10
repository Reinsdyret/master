= Experiments and results <ch:results>
In this section we define the simulations we designed, algorithms to run on these simulations and metrics used to compare them. 
Then we present our hypotheses we had before running the experiments.
Finally we present and discuss results from the simulations.

== Experimental setup
Because of privacy and time constraints we could not use real data from Helfo.
We then proceeded to use randomly generated data, while trying to keep relative sizes compared to real numbers in Norway.
In the following sections we define how the data generation is done, our simulation model, exactly what algorithms we compared and the metrics we use to measure them.

=== Data generation
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

=== Simulation model
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

=== Algorithms compared
We have in total five algorithms to compare:
- _Huitfeldt TTC_
- _Greedy DFS_
- _Cycle Cancelling for cardinality_
- _Cycle Cancelling for utility_
- _Cycle Cancelling for strict priority_

These algorithms we each ran in a 10 year simulation, with a population of 

In addition we run four variations of the _Cycle Cancelling for utility_ with different base for the priority function.


=== Metrics


== Results

=== Waitlist size over time 

=== Maximum waiting time 

=== Average waiting time 

=== Runtime 

== Discussion
