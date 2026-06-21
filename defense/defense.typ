#import "@preview/diatypst:0.9.3": *
#show: slides.with(
  title: "Patient Reallocation and Waitlist
Reduction in the Norwegian GP
System", // Required
  subtitle: "Ikke hvor mange, men hvilke?",
  date: "26.06.2026",
  authors: ("Lars Møen Haukland"),
  title-color: rgb("#cf3c3a")
)

= Intro

== Motivasjon
- Fastlegeordningen (2001)
- Lang kø
  - Bergen 97%
  - 274 dager i 2024 
- Ansetter flere fastleger

#include "figs/antall_venteliste.typ"

== Løsningen?
- Redusere antall folk på venteliste -> Flere folk er fornøyd?

#include "figs/doktor-eksempel.typ"

== Bakgrunn
- Agenter og Objekter
- Lignende problemer
  - Housing Market
  - Kidney exchange
  - College Admissions problem

- Top Trading Cycles algoritmen

== Top Trading Cycles (TTC)
- $A = [h_B, h_C, h_A]$
- $B = [h_C, h_A, h_B]$
- $C = [h_B, h_C, h_A]$
\
\
\
#include "figs/ttc-graph.typ"
\
\
\
\
- $A = [h_B, h_C, h_A]$
- $B = [h_C, h_A, h_B]$
- $C = [h_B, h_C, h_A]$
#include "figs/ttc-graph-2.typ"

- Strategi trygt
- Individuell rasjonalitet
- Pareto effektiv

== Huitfeldt et al. TTC
- Pasienter har prioritet 
  - Ventetid


= GP allocation problem

== Definisjon

== Løsninger

== Leksikografisk optimalitet

== Kardinalitet

== Nytte (utility)

== Prioritetsfunksjon


= Algoritmer

== Top Trading Cycles (TTC)

== Cycle Cancelling

== Cycle Cancelling for kardinalitet

== Cycle Cancelling for Nytte

== Cycle Cancelling for leksikografisk optimalitet


= Eksperimenter og resultater

== Simulering og data

== Metrikker
- Antall hjulpet
- Størrelse på ventelisten
- Vente tid
  - Maks
  - Gjennomsnitt
- Kjøretid

== Resultater
- Halaa

== Konklusjon
temp


