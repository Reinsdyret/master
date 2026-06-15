#import "lib.typ": *

#set text(lang: "en")

#show: conf.with(
  title: [Patient Reallocation and Waitlist Reduction in the Norwegian GP System],
  author: "Lars Møen Haukland",
  supervisors: "Fredrik Manne",
  institution: "University of Bergen - Department of Informatics",
  institution-figure: "figs/NT_PositivVenstrestilt_ENG.svg",
  date: datetime(year: 2026, month: 6, day: 15),
  abstract: [
    Many assignment problems share a common structure: each agent already holds one item, such as a house, a school place, or a doctor, and some agents would prefer to hold an item currently held by someone else.
    When no spare items are available, the only way to satisfy these agents is to find cycles of agents who can exchange their items simultaneously.
    This thesis studies how to find such exchanges, motivated by the allocation of general practitioners (GPs) in Norway, where people wanting to switch GP must wait for a slot to open because almost all GP lists are full.
    We formalise the problem as one of finding cycles in a directed graph of patients and GPs, and define three notions of a good set of exchanges, one that resolves as many patients as possible, one that gives strict priority to those who have waited the longest, and one that balances the two.
    We develop exact algorithms based on cycle cancelling for each of these, together with a fast heuristic, and compare them against an existing mechanism in a simulation of the Norwegian GP system.
    We find that the choice of objective has little effect on how many patients are resolved, but a large effect on which patients are resolved and therefore on how long they wait, and that the algorithms differ widely in running time.
  ],
  acknowledgements: [
    I would like to thank my supervisor Fredrik Manne for his guidance, support and patience throughout this project.
    I would also like to thank Juni Weisteen Bjerde for new perspectives on the topics we discussed.
    In addition I thank Mathias Berntert for his input on the GP problem and on cycle cancelling in particular.
    My thanks also go to Ingrid Huitfeldt, Victoria Marone and Daniel Waldinger, whose work developing and testing their Top Trading Cycles mechanism inspired us to explore this problem.
    Their sharing of the replication package also made it possible to compare our algorithms against theirs, which was invaluable for my thesis.
  I am grateful to Tommy Odland for taking the time to help me understand the Top Trading Cycles mechanism and other related mechanisms.

    I am also very appreciative of my fellow students, who made my degree not only more insightful but also fun and memorable.
    Ljubo, Jacob and Sindre in particular made my time in the reading hall far more enjoyable, with fun and interesting conversations, long breaks, and intense rounds of table tennis that finally helped me surpass my father's skill at the game.

    I thank my family for always supporting and comforting me, and for making sure I did not forget I had a thesis to work on.

    Lastly I would like to thank my girlfriend, Kristine, for her patience, love and support.
    Sometimes coming home to a warm dinner after being confused all day was just what I needed.
  ],
  appendix: (
    enabled: true,
    title: "Appendix",
    heading-numbering-format: "A.1.1",
    body: include "chapters/appendix.typ",
  ),
  bibliography: bibliography("refs.bib"),
  figure-index: (enabled: true),
  table-index: (enabled: true),
  listing-index: (enabled: true),
)

#include "chapters/01-introduction.typ"
#include "chapters/02-background.typ"
#include "chapters/03-problem.typ"
#include "chapters/04-implementation.typ"
#include "chapters/05-results.typ"
#include "chapters/06-conclusion.typ"
