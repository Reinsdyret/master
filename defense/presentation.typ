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

// FORSLAG (valgfritt bilde): et rolig foto av et norsk legekontor / venterom på
// tittel- eller motivasjonssliden for å humanisere problemet. Mange CS-forsvar
// holder seg til rene diagrammer, så dette er kun hvis du vil ha litt varme.

= Intro
== Motivasjon
- Fastlegeordningen (2001)
- Lang kø
  - Bergen 97%
  - 274 dager i 2024 
- Ansetter flere fastleger
#include "figs/antall_venteliste.typ"
== Løsningen?
nevn dette med hvordan køene funker til nå
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
- Strategi-trygt
- Individuell rasjonalitet
- Pareto-effektiv
== Huitfeldt et al. TTC
- Pasienter har prioritet 
  - Ventetid
- Flere pasienter kan eie samme ønskede fastlege → $"Top"(i)$ ikke entydig
- Først et ventelistesteg (fyller ledige slots), så TTC på resten
- Mister strategitrygghet når den kjøres gjentatte ganger

= The GP allocation problem
== Definisjon
- Input:
  - Pasienter $P$, fastleger $D$
  - $D_"cur"[i]$ – nåværende fastlege, $D_"pref"[i]$ – ønsket fastlege
  - Prioritetsfunksjon $R: P -> NN$ (høyere $=$ viktigere)
- Mulig løsning (feasible):
  - Pasientene i $S$ bytter slots samtidig
  - Alle i $S$ ender hos sin ønskede fastlege
  - Antall pasienter per fastlege er uendret
- Kant $(a, b)$: pasient $a$ kan overta plassen til $b$
- $S$ er mulig $<=>$ $G_S$ dekkes av vertex-disjunkte sykler
// FORSLAG (bilde): liten bipartittgraf pasient/fastlege med én sykel markert.
// Du kan trolig gjenbruke / tilpasse doktor-eksempel.typ.
== Løsninger
- Når er én løsning bedre enn en annen?
- En ordning $succ$ over mulige løsninger
- Optimal: ingen mulig løsning er større under $succ$
- Tre varianter:
  - Leksikografisk – etter prioritet
  - Kardinalitet – etter antall
  - Nytte (utility) – en kombinasjon
== Leksikografisk optimalitet
- Pasienter sortert etter prioritet (høyest først), $R(p_i) = i$
- Karakteristisk vektor $chi(S) in {0, 1}^n$
- $S succ_"lex" S'$: $chi(S)$ er leksikografisk større enn $chi(S')$
- Som et binærtall – mest signifikante bit $=$ høyest prioritet som hjelpes
- Hjelp alltid den høyest prioriterte du kan, så nest høyeste, ...
- Tie-break avgjør ved lik prioritet
\
- Eksempel: $S = {p_4, p_5}$ vs. $S' = {p_1, p_2, p_3}$
- $chi(S) = (1, 1, 0, 0, 0)$, $chi(S') = (0, 0, 1, 1, 1)$ → $S succ_"lex" S'$

#include "figs/example-graph.typ"

// FORSLAG (bilde): eksempelgraf G (Figur 5 i oppgaven).
== Kardinalitet
- Flest mulig pasienter byttet
- $S succ_"size" S'$ hvis $|S| > |S'|$
- Ingen hensyn til prioritet – alle teller likt
\
- Eksempel (samme graf):
  - $|S| = 2$, $|S'| = 3$ → $S' succ_"size" S$
  - Optimalt: ${p_1, p_2, p_3, p_4, p_5}$ (begge sykler)
- Merk: $S succ_"lex" S'$ men $S' succ_"size" S$ – motsatt rekkefølge!
// FORSLAG (bilde): samme eksempelgraf G (Figur 5).
== Nytte (utility)
- Mellomting mellom leksikografisk og kardinalitet
- Total nytte: $U(S) = sum_(p in S) R(p)$
- $S succ_"util" S'$ hvis $U(S) > U(S')$
- Hver pasient teller, vektet med prioritet
\
- Eksempel: $U(S) = 1 + 3 + 4 = 8$ vs. $U(S') = 2 + 5 = 7$ → $S succ_"util" S'$
- Mange lavprioriterte kan slå få høyprioriterte
#include "figs/example-graph-2.typ"
// FORSLAG (bilde): eksempelgraf G (Figur 6).
== Prioritetsfunksjon
- Stor effekt på hvilke pasienter som faktisk hjelpes
- $R(a) = k^("dager ventet")$, med $1 <= k <= 2$
- $k = 1$: alle får prioritet 1 → nytte $=$ kardinalitet
- $k = 2$: høyprioritert slår alle under seg → nytte $approx$ leksikografisk
// FORSLAG (anbefalt hero-figur): "spekter"-illustrasjon, se forklaring i chatten.
// #include "figs/spekter.typ"

= Algoritmer
== Top Trading Cycles (TTC)
- Baseline vi sammenligner mot
- Direkte port fra Python til Rust (lik kjøretidssammenligning)
- Alle fastleger fulle → ventelistesteget hopper over
== Greedy DFS (heuristikk)
- Rask og enkel heuristikk som fortsatt respekterer prioritet
- Inspirert av TTC, men lar fastlegen peke til ALLE pasientene sine
- Gå gjennom pasienter i synkende prioritet, kjør DFS for å finne en sykel
- Sykel funnet → løs den med en gang og fjern pasientene
- Grådige valg → ikke garantert optimal
- $O(|P| (|D| + |P|))$


#include("figs/pareto-inefficient.typ")
// FORSLAG (bilde): suboptimalt eksempel for Greedy DFS (Figur 7) – veldig illustrativ.
== Cycle Cancelling
- Teknikk for minimum-cost circulation
- Residualnettverk – ledig kapasitet + flyt som kan angres
- Negative Cycle Optimality Theorem:
  - Optimal $<=>$ ingen negativ sykel i residualnettverket
- Algoritme: start $f = 0$; så lenge negativ sykel finnes → kanseller den
- Bellman-Ford finner syklene, $O(n m^2 C U)$ (pseudo-polynomisk)
// FORSLAG (bilde): sirkulasjonsnettverk-eksempel (Figur 4) eller et lite residualgraf-diagram.
== Cycle Cancelling for kardinalitet
- GP collapsed graph – kun fastleger som noder
- Hver kant: kostnad $-1$, kapasitet $=$ antall pasienter som vil bytte
- Min kostnad $=$ mest flyt $=$ flest pasienter
- Polynomisk: $C = 1$, $U <= |P|$ → $O(n m^2 |P|)$
== Cycle Cancelling for Nytte
- GP multigraph – hver pasient er én egen kant
- Kapasitet $1$, kostnad $-R(p_i)$
- Min kostnad $=$ optimal under $succ_"util"$
- Kjøretid $O(n m^2 max R(p))$ – polynomisk hvis prioritet er polynomisk begrenset
- Eksponentiell prioritet: 128-bit heltall, buckets på 10 dager 
== Cycle Cancelling for leksikografisk optimalitet
- GP multigraph, men uten kostnader
- Løp gjennom pasienter i synkende prioritet
- Finnes sti $D_"pref"[i] -> D_"cur"[i]$ i residualgrafen?
  - Ja → push én enhet flyt, legg pasienten til i løsningen
- Slett kanten → beslutningen er endelig
- Monotonicity of Feasibility – hvorfor det er trygt å slette
- $O((n + m) |P|)$ – raskere enn Bellman-Ford, håndterer tie-breaks eksakt
== Egenskaper
- Individuell rasjonalitet: alle algoritmene (ingen flyttes dit de ikke vil)
- Pareto-effektiv: ingen kan forbedres uten å forverre en annen
- Strategitrygghet:
  - Én kjøring: ja (sannferdig rapportering er dominant)
  - Gjentatt: usikkert – prioritet avhenger av ventetid

= Eksperimenter og resultater
== Simulering og data
- Syntetiske data (personvern + tid), proporsjoner matchet til Norge
- 100 000 pasienter, 102 fastleger (~980 per lege)
- 5% startventeliste, 6 distrikter (~17 leger hver)
- 11% forespørsler på tvers av distrikt, 18 nye forespørsler/dag
- Dag for dag:
  - Prioritet og ventetid $+1$
  - Kjør algoritmen → flytt løste pasienter
  - Legg til nye forespørsler
- 100 år (36 500 dager) + 2 år for de eksponentielle variantene
== Metrikker
- Antall hjulpet
- Størrelse på ventelisten
- Vente tid
  - Maks
  - Gjennomsnitt
  - 99-persentil
- Andel på tvers av distrikt
- Kjøretid
== Resultater: Antall hjulpet
- Alle algoritmene løser nesten like mange (~1% forskjell)
  - 622 002 (strict priority) – 629 497 (cardinality)
- Spørsmålet er ikke *hvor mange*, men *hvilke*
// FORSLAG (bilde): Figur 8 – total resolved per algoritme.
== Resultater: Ventelistestørrelse
- Cardinality lavest, strict priority høyest
- Overraskelse: Huitfeldt TTC nesten på linje med cardinality
- Listene klatrer hele tiden – lukket modell (se forbehold)
// FORSLAG (bilde): Figur 9 – waitlist size over tid (100 år).
== Resultater: Ventetid
- Cardinality & Huitfeldt TTC: kortest snitt, men noen venter nesten "evig"
- Strict priority: lengst snitt, men best verste-tilfelle
- Snitt: Huitfeldt 352.9 d, cardinality 940.7 d, resten > 1300 d
- Maks: cardinality & TTC når 36 500 d (aldri løst); resten er begrenset
- 99-persentil: cardinality verst (14 219 d), strict priority best (9 335 d)
// FORSLAG (bilde): Figur 11 (99-persentil + maks) og/eller Figur 13 (snitt).
== Resultater: Kjøretid
- Cardinality klart raskest: 9.2 ms/dag
- Resten: 128–379 ms/dag (én grafsøk per pasient → tregest)
- Grunn: collapsed graph skalerer med antall fastleger, ikke pasienter
// FORSLAG (bilde): Figur 15 – kjøretid per dag.
== Resultater: 
- Samme antall hjulpet – men ulike pasienter
- Gjennomstrømning (kort snittventetid) vs. rettferdighet (begrenset maks)
// FORSLAG (bilde, valgfritt): Figur 17 – på tvers av vs. innen distrikt.
== Konklusjon
- Formaliserte GP allocation-problemet som sykler i en rettet graf
- Tre optimalitetskriterier – nytte har kardinalitet og leksikografisk som endepunkter
- Eksakte cycle cancelling-algoritmer + rask Greedy DFS-heuristikk
- Hovedfunn: valg av mål endrer nesten ikke *hvor mange*, men *hvilke* – og dermed ventetiden
- Avveining: gjennomstrømning vs. verste-tilfelle-rettferdighet
- Uventet: TTC-baselinen holder følge med cardinality på ventelistestørrelse
== Videre arbeid
- Ekte data fra Helfo
- Åpen modell med ledig kapasitet + et ventelistesteg før byttene
- Gjentatte kjøringer med konfidensintervall
- Strategisk oppførsel i gjentatt setting
- Sterkt polynomisk metode (Goldberg–Tarjan) / begrenset sykellengde / topartspreferanser
== Takk for meg!
- Spørsmål?
