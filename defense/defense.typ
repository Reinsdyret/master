#set text(lang: "nn")
#import "@preview/diatypst:0.9.3": *
#show: slides.with(
  title: "Patient Reallocation and Waitlist
Reduction in the Norwegian GP
System", // Required
  date: "26.06.2026",
  authors: ("Lars Møen Haukland"),
  title-color: rgb("#761a19"),
  toc: false,
)

= Intro

== Motivasjon
- Fastlegeordningen (2001)
- Vil bytte fastlege til en fastlege med full kapasitet -> venteliste.
- Når det åpner seg ledig kapasitet blir den tildelt pasient som har ventet lengst.
- 77% av alle fastleger med full kapasitet, 2025.
  - Bergen 97%
  - Gjennomsnittlig ventetid på 274 dager i 2024, median 137 dager. 

#include "figs/antall_venteliste.typ"

== Løsningen?
- Finne bytter mellom pasienter på venteliste.
- 2 pasient bytter eller større ubegrenset lengde sykler.
#include "figs/doktor-eksempel.typ"

== Ideen
- Bruke algoritmer for å finne sykler i ventelistene.
- Redusere antall pasienter på ventelistene.
- Redusere ventetid for pasienter på ventelistene.

- Testet algoritmene mot hverandre i simuleringer.
- Analysert hvordan algoritmer med forskjellig mål påvirker resultatene.

= Bakgrunn

== Lignende problemer
- _Assignment_ problemer
- Agenter og objekter - pasienter og leger
- One-to-one
- Many-to-one
- Housing Market
- Kidney exchange
- College Admissions problem

== _Top Trading Cycles_ (TTC)
- Algoritme utviklet for å reallokere objekter slik at alle bytter agenter kan gjøre for å få sitt foretrukne objekt blir utført.
- Hver agent har fullstendig prioriteringsliste over alle objekter, sortert etter mest foretrukne.

- Lag en graph $G = (V,E), V = "agenter"$
- Lag en kant fra hver agent $u$ til den agenten som eier $u$ sitt mest foretrukne objekt, noteres som $"Top"(u)$.
- Finn og utfør en sykel, fjern nodene i sykelen fra grafen
- Gjenta med å oppdatere kantene, finn sykel, helt til det ikke er noen noder

== _Top Trading Cycles_ (TTC)
- $A = [h_B, h_C, h_A]$
- $B = [h_C, h_A, h_B]$
- $C = [h_B, h_C, h_A]$
#v(1.5em)
#align(center)[
  #include "figs/ttc-graph.typ"
  #v(2em)
  #include "figs/ttc-graph-2.typ"
]

== Egenskaper til TTC
- Strategitrygt
- Individuell rasjonalitet
- Pareto-effektiv

== TTC for ventelistene
- Utviklet av Huitfeldt et al.
- Hver pasient har en prioritet.
  - f.eks ventetid eller en funksjon av ventetid.
- $G = (V,E), V = "pasienter" union "leger"$.
- Siden flere pasienter kan ha samme lege, er ikke $"Top"(u)$ bare en pasient.
  - Hver pasient har en kant til sin foretrukne lege.
  - Hver lege har en kant til pasienten sin med høyest prioritet.


= GP allocation problem

== Definisjon
- Input
  - P: Sett av pasienter
  - D: Sett av leger
  - R: Prioritetsfunksjon $R: P -> NN$
  - 

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


