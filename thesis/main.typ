#import "lib.typ": *

#set text(lang: "en")

#show: conf.with(
  title: [Top Trading Cycles in a practical setting],
  author: "Lars Møen Haukland",
  supervisors: "Fredrik Manne",
  institution: "University of Bergen - Department of Informatics",
  institution-figure: "figs/NT_PositivVenstrestilt_NOR.svg",
  date: datetime(year: 2026, month: 6, day: 1),
  abstract: [
    This thesis explores the Top Trading Cycles algorithm in the practical setting of the allocations of GPs, General Practitioner, in Norway for people wanting to switch or get a GP. It explores how different focus of _good_ results can change both the runtime and results, and how to make the algorithm run as efficiently as possible. This thesis proposes an implementation of the Top Trading Cycles that can reduce the waitlists for GPs in norway by over 80%.
  ],
  acknowledgements: [
    I would like to thank my supervisor Fredrik Manne for his guidance and support throughout this project.
    I would also like to thank Juni Weisteen Bjerde for input during this project.
  ],
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
