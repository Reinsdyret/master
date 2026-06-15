= Cross district request outcomes

Since none of the algorithms are aware of districts, the district structure we
added for realism could in principle leave cross district requests, where fewer
patients want to switch in the opposite direction, systematically worse off. The
figure below shows it does not.

#figure(
  image("../figs/images/district_cross_plot.svg", width: 100%),
  caption: [
    Cross district and within district request outcomes under each algorithm, in
    the main 100 year simulation. Left: the realization rate, the
    share of added requests eventually resolved. Right: the longest wait among
    resolved patients.
  ],
) <fig-district>


= Statement on the Use of AI Tools

Claude was used throughout the thesis to provide structure, grammar correction, spelling fixes and discussing ideas with.
Grammarly was also used for this same purpose.
Claude Code was used as a programming assistant, but only when I knew exactly what i wanted done and how. 
It helped implement the python scripts to generate random data, plot results and other small scripts.
In addition it helped implement the simulation and locate and fix bugs.
All work these AI tools did i was aware of and I checked their work and made sure this is what i wanted.
I am aware that I am responsible for all content of this master’s thesis.
