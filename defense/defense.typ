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
  count: none
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
  - $P$: Sett av pasienter
  - $D$: Sett av leger
  - $R$: Prioritetsfunksjon $R: P -> NN$
  - $D_"cur"[i]$, $D_"pref"[i]$: lister som sier nåværende lege og foretrukne lege for pasient $i$
- Løsning 
  - $S subset.eq P$: Sett av pasienter, slik at $G_S$ kan dekkes av ikke-overlappende sykler.
$
G_S = (S, E = {(a,b) | a,b in S, "foretrukne legen til" a "er nåværende lege til "b})
$

#include "figs/example-feasible-solution.typ"
#include "figs/example-feasible-solution-2.typ"
  
== Optimal løsning
- Definerer et generell optimeringskriterium $succ$
- en rangering av løsninger
- optimal løsning dersom ingen annen løsning er mer maksimal under $succ$
- Lager varianter av dette

== Leksikografisk optimalitet
- $succ_"lex"$
- Sorter pasienter basert på prioritet slik at: $R(p_1) >= R(p_2) >= dots >= R(p_n)$, ($n = |P|$)
- $ chi(S) = (b_1, b_2, dots, b_n) in {0,1}^n, quad b_i = cases(1 & "if" p_i in S, 0 & "otherwise")$

#import "@preview/ctheorems:1.1.3": *
#show: thmrules.with(qed-symbol: $square$)
#let theorem = thmbox("teorem", "Teorem", fill: rgb("#eeffee"))
#let corollary = thmplain(
  "corollary",
  "Corollary",
  base: "theorem",
  titlefmt: strong
)
#let definition = thmbox("definisjon", "Definisjon", inset: (x: 1.2em, top: 1em))

#definition("Leksikografisk rangering fra prioritet")[
  $
  S succ_"lex" S' quad <=> quad chi(S) "er leksikografisk mer maksimal enn" chi(S')
  $
  Så, på første index $i$ hvor $S$ og $S'$ er forskjellig, $chi(S)_i = 1$ og $chi(S')_i = 0$.
]

#include "figs/lexicographic-example.typ"


== Kardinalitet
- $succ_"size"$
- Størrelsen på løsningnen

#definition("Kardinalitet rangering")[
  $
  S succ_"size" S' <=> |S| > |S'|
  $
]

#include "figs/cardinality-example.typ"

== Nytte
- $succ_"util"$
- $U(S) = sum_(p in S) R(p)$
- Prioritet teller, men kan droppe en høy prioritet pasient for flere lavere.

#definition("Nytte rangering")[
  $
  S succ_"util" S' <=> U(S) > U(S')
  $
]

#include "figs/util-example.typ"

== Prioritetsfunksjon
- Bestemmer direkte hvilke pasienter som blir inkludert i en maksimal løsning i $succ_"util"$ og $succ_"lex"$
- Lett å basere på antall dager
- $"wait"(a) = "antall dager a har vært på venteliste"$
- Lineær: $R(a) = "wait"(a)$
- Eksponentiell: $R(a) = k^("wait"(a))$
  - $k$ bestemmer hvor mye prioritet er vektet
  - $k=1$ alle prioriteter er 1 $arrow succ_"util"$ blir $succ_"size"$
  - $k=2$ hvis $"wait"(a)$ er unik for alle pasienter $arrow succ_"util"$ blir $succ_"lex"$


= Algoritmer

== Top Trading Cycles (TTC)
- Tatt fra replikasjonspakken
- Python oversatt til rust
  - Kjøretid ikke påvirket av språk

== Greedy DFS 
- $G = (V, E), quad V = P union D$
- $E = {(p_i, D_"pref"[i])} union {(D_"cur"[i], p_i)}$
  - Pasienter peker på sin foretrukne lege
  - Leger peker på alle sine pasienter

- For hver pasient i synkende prioritet
  - Prøv å finn en sykel med denne pasienten, DFS
    - På hver lege gå til pasient med høyeste prioritet først
  - Gi alle pasientene i sykelen sin foretrukne lege og fjern fra grafen

- Hvis ingen sykel finnes for en pasient, kan den markeres som "stuck"

- $O(|P| (|D| + |P|))$

#include "figs/pareto-inefficient.typ"

== _Minimum Cost Circulation_
- Input: $G = (V,E)$ hvor G er en rettet multigraf
- Kantene er tripler: $(v,w,i)$
  - Har kostnad $c(v,w,i) in ZZ$
  - Og kapasitet $u(v,w,i) >= 0$

- Finne en circulation $f$ som setter verdi på hver kant som tilfredstiller:
$
0 <= f(v,w,i) <= u(v,w,i) quad forall (v,w,i) in E
$
$
sum_((q,w,i) in "in"(w)) f(q,w,i) = sum_((w,v,j) in "out"(w)) f(w,v,j) quad forall w in V
$

$
"cost"(f) = sum_((v,w,i) in E) c(v,w,i) f(v,w,i)
$

- Finne circulation $f^*$ med minst kostnad

== Residual graf
#grid(
  columns: (1.4fr, 1fr),
  column-gutter: 1em,
  align: (left, center + horizon),
  [
    - Omskrivning av grafen basert på $f$
    - Nodene endres ikke fra original grafen
    - For hver kant $(v,w,i)$ lag to kanter
      - $(v,w,i)$ fremover kant
        - samme kostnad som i original
        - $u_f(v,w,i) = u(v,w,i) - f(v,w,i)$
      - $(w,v,i)$ baklengs kant
        - $-c(v,w,i)$
        - $u_f(w,v,i) = f(v,w,i)$
  ],
  include "figs/residual-graph-example.typ",
)

== Optimal circulation 
#theorem("Negative Cycle Optimality Theorem")[
  En circulation $f^*$ er en optimal løsning til _Minimum Cost Circulation_ problemet hvis og bare hvis residual grafen $G(f^*)$ ikke har noen negativ-kostnad rettede sykler.
]<negative-cycle-optimality-theorem>

- I en instanse med bare positive kostnader, $f^*(e) = 0 quad forall e in E$
- Vi setter kostnadene til å være negative, da finner vi også løsningen med maksimal total circulation
$
sum_((v,w,i) in E) f(v,w,i)
$

== _Cycle Cancelling_
- Starter med 0 circulation: $f^0(e) = 0 quad forall e in E$
- Så lenge det finnes en negativ sykel i residual grafen
  - Kanseller den: Øk flyt på hver kant i sykelen med den minste kapasiteten til en av kantene i sykelen

#include "figs/cc-1.typ"
#include "figs/cc-2.typ"
#include "figs/cc-3.typ"

== _Cycle cancelling_ kjøretid 
- $O(n m^2 C U)$, $n = |V|, m = |E|$
  - $C$: maks kostnad på en kant 
  - $U$: maks kapasitet på en kant
- Pseudo-polynom
  - For en av våre algoritmer setter vi kostnad basert på $R$

- _Mean Cycle Cancelling_

== _Cycle Cancelling_ for kardinalitet
- $G = (V,E), quad V = D$
- Kant $(v,w,i)$ dersom det finnes minst en pasient som vil bytte fra lege $v$ til lege $w$
- Kostnad $c(v,w,i)$ er $-1$
- Kapasitet er antall pasienter som vil ha det byttet
  - $u(v,w,i) = 5$: fem pasienter vil bytte fra v til w 
\
- Grafen lagrer ingenting om prioritet
- Etter _Cycle Cancelling_ har kjørt på grafen får vi bare vite hvor mange som kan byttes på hver kant
  - Hvis $k$ pasienter kan byttes, velge de $k$ som har ventet lengst
- Negativ kostnad gjør at vi finner optimal løsning under $succ_"size"$
\
- Kjøretid: $O(n m^2 C U) arrow O(n m^2 |P|)$ 

== _Cycle Cancelling_ for Nytte
- $G = (V,E), quad V = D$
- Kant $(v,w,i)$ dersom det finnes én pasient $i$ som vil fra lege $v$ til $w$
  - En kant per pasient
- Kostnad $c(v,w,i) = - R(p_i)$
- Kapasitet $u(v,w,i) = 1$ én pasient per kant
\
- Etter _Cycle Cancelling_ har kjørt på grafen har vi flyt på noen kanter som betyr at pasientene som tilhører kan få sin foretrukne lege.
- Optimal løsning under $succ_"util"$
- Kjøretid: $O(n m^2 C U) arrow O(n m^2 "max" R(p))$
  - pseudo-polynom
- Max prioritet i våre simuleringer
  - Lagret som 128 bit heltall. $R(a) = 2^("wait"(a) / 10)$
  - Maks 1010 dager


== _Cycle Cancelling_ for $succ_"lex"$
- Bruker fortsatt cycle cancelling teknikken men med litt endring
- $G = (V, E), V=D$
- Kant $(v,w,i)$ dersom det finnes én pasient $i$ som vil fra lege $v$ til $w$.
- Bryr oss ikke om kostnad
- Kapasitet $u(v,w,i) = 1$
\
- For hver pasient $p_i$ i synkende prioritet 
  - La $e = (v,w,i)$ være kanten til $p_i$
  - Hvis $f(e) = 1$: Legg $p$ til i løsningen og slett $e$ og residual kanten dens fra grafen
  - Ellers:
    - Hvis det finnes en sti $Q$ fra $w$ til $v$ i $G(f)$, bruker BFS
      - Det finnes sykel som inneholder $p_i$ da må den være med i løsningen
      - Øk flyt med en på hver kant i $Q$, legg $p_i$ til løsningen og slett $e$ og residual kanten dens

#include "figs/cc_lex-1.typ"
#include "figs/cc_lex-2.typ"
#include "figs/cc_lex-3.typ"
#include "figs/cc_lex-4.typ"

- Kjøretid: $O((n + m) |P|)$
  - Går over alle pasienter: $|P|$ iterasjoner
    - For hver pasient så søker vi med BFS $(n + m)$ iterasjoner

= Eksperimenter og resultater

== Simulering og data
- Ikke mulighet for ekte data
  - Tilfeldig generert
- Simulerer en mindre instans men proporsjonell til norges tall.
- To simuleringer
  - 100 år: se utviklingen av ventelistestørrelse 
  - 2 år: Inkludere eksponentielle prioriteringsfunksjoner
\
- For hver dag:
  - Øke ventetiden til pasienter med 1
  - Kjøre algoritmen og fjerne pasienter som løses 
  - Legge til nye pasienter på ventelister

== Metrikker
- Antall hjulpet
- Størrelse på ventelisten
- Vente tid
  - Maks
  - Gjennomsnitt
- Kjøretid

== Antall hjulpet
- Lite variasjon
#figure(
  image("figs/images/antall_hjulpet.svg", width: 70%),
  caption: [Totalt antall legebytter utført],
)

== Størrelse på ventelisten 
#figure(
  image("figs/images/størrelse_på_venteliste_100y.svg"),
  caption: [Totalt antall pasienter på venteliste over tid]
)

== Maks vente tid 
#figure(
  image("figs/images/summary_resolved_p99_overall_max_100y.svg"),
  caption: [Maks ventetid for løste bytter, uten de 1% lengste på venstre siden og totalt max ventetid inkludert pasienter som ikke fikk byttet på høyre.]
)

== Gjennomsnittlig vente tid 
#figure(
  image("figs/images/summary_resolved_avg_wait_100y.svg", width: 80%),
  caption: [Gjennomsnittlig ventetid for bytter som ble løst]
)

#figure(
  image("figs/images/summary_resolved_avg_wait_2y.svg")
)

== Kjøretid 

== Konklusjon
- Ikke representabel simulering ift Norge.
  - Ingen pasienter som dør eller blir født.
  - Økende venteliste ville ikke skjedd i et reelt system.
  - Først kjøre ventelistesteg hvor alle ledige plasser blir gitt.
- Viser fortsatt hvordan algoritmenes prioriteringer endrer resultat.
Videre arbeid:
- Teste med ekte data 
- Dynamisk algoritme, maksimerer $succ_"size"$ så lenge det ikke er noen pasienter som har ventet i mer enn $x$ dager, ellers maksimer $succ_"lex"$.
- Analysere om noen kan utnytte algoritmene for å få en urettferdig fordel.
- Bruke _Mean Cycle Cancelling_ for polynomisk kjøretid for _Cycle Cancelling for Nytte_

= Takk for meg, spørsmål?

