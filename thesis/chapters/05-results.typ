= Experiments and results <ch:results>
In this section we define the simulations we designed, algorithms to run on these simulations and metrics used to compare them. 
Then we present our hypotheses we had before running the experiments.
Finally we present and discuss results from the simulations.

== 5.1 Experimental setup
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
Based on these weights we distribute patients among doctors, the higher a doctors weight the more patients that doctor gets. We make sure each doctor gets at least one patient.


=== Simulation model

=== Algorithms compared

=== Metrics


== Results

=== Waitlist size over time 

=== Maximum waiting time 

=== Average waiting time 

=== Runtime 

== Discussion
