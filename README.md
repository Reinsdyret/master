# master

## Partisjonering av noder i en graf i sykler og stier 

Et stort problem i mange situasjoner er tildeling. Om det er tildelinger av studentplasser på lesesaler, fastleger til privat personer eller skoler til studenter. Selve tildelingen er godt studert men noe som er litt mindre sett på er dynamisk tildeling. Altså når alle agentene har blitt tildelt noe men så vil de bytte. For eksempel en student vil sitte på en annen lesesal. Den enkleste løsningen da er å lage en kø hvor første man som stiller seg i køen for en lesesal får plassen når det blir en ledig plass. Men denne snarveien fører til flere problemer. Nemlig hvis to studenter vil bytte lesesal så må de fortsatt stå i køen og vente på at en plass blir ledig før de kan bytte. Selvom de to kunne bli tatt ut av køen og bare byttet plass så vil kø systemet gjøre at de må vente som på en større skala kan føre til veldig overflødige kø størrelser.

Jeg vil skrive om algoritmer som kan identifisere slike situasjoner og tildele slik at flest agenter blir tilfredsstilt. Jeg vil og utforske implementeringer av slike algoritmer og sammenligne de.


