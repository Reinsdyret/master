// Get Polylux from the official package repository
#import "@preview/polylux:0.4.0": *

// Make the paper dimensions fit for a presentation and the text larger
#set page(paper: "presentation-16-9")
#set text(size: 25pt, font: "Lato")

// Use #slide to create a slide and style it using your favourite Typst functions
#slide[
  #set align(horizon)
  = Optimizing Patient-Doctor Matching in Norwegian GP Systems

  Lars Haukland

  February 19, 2023
]

#slide[
  == Topics
  - Me
  - Problem
  - TTC approach
  - Cycle cover
  - Metaheuristic approach
]

#slide[
  == Me
  #uncover("1-")[- I am from Stavanger]
  #uncover("2-")[- Always wanted to code]
  #uncover("3-")[- Did my bachelor at UiB, DataTeknologi]
  #uncover("4-")[- Had INF234 with Pål and enjoyed the topics]
  #uncover("5-")[- My advisor is Fredrik Manne]
  #uncover("6-")[
  - My biggest strength:
    - I love trying out new hobbies and activities
  ]
  #uncover("7-")[
  - My biggest weakness:
    - I spend way too much money on my new hobbies
  ]
]

#slide[
  #grid(
    columns: (1fr, 1fr, 1fr),
    gutter: 1em,
    align(center)[#image("me-skiing.jpg", height: 350pt, fit: "contain")],
    align(center)[#image("me.jpg", height: 350pt, fit: "contain")],
    align(center)[#image("bergen-voss-2025.jpg", height: 350pt, fit: "contain")],
  )
]

#slide[
  #grid(
    columns: (1fr, 1fr, 1fr),
    gutter: 1em,
    align(center)[#image("bjj.jpg", height: 350pt, fit: "contain")],
    align(center)[#image("ping-pong.jpg", height: 350pt, fit: "contain")],
    align(center)[#image("skate.png", height: 350pt, fit: "contain")],
  )
]

#slide[
  == Two true and one false

  - I cycled from Bergen to Voss in under 6h
  - Fredrik was also my dads master advisor
  - I have reduced from 3-SAT to hamiltonian path
]

#slide[
  == The problem
  How HelseNorge handles when someone wants to switch their GP:
  - First come first serve 
  - When full, queue up
  
  #grid(
    columns: (1fr, auto),
    column-gutter: 1em,
    [
      #uncover("1-")[
        - Lars has Doctor A and Jonas has Doctor B.
      ]
      #uncover("2-")[
        - But Lars wants Doctor B and Jonas wants Doctor A.
      ]
      #uncover("3-")[
        - Now Lars and Jonas have to wait in line for a space to open up
      ]
    ],
    image("example1.png", height: 200pt)
  )
]

#slide[
  #image("example2.png")
]

#slide[
  #image("example3.png")
]


#slide[
  == What if they could swap
  - No waiting in line
  - There are fewer people in line
  - Both are happy
]

#slide[
  == How do we find swaps
  We look for cycles

  - Two variations
   - Priority
   - Maximizing total switches

  - Essentially just trying to find a set of cycles $S$ that cover as many vertices in $G$

]

#slide[
  == First algorithm

  - Top trading cycles
  - DFS from highest priority always following higher priority 
]

#slide[
  == Connection to cycle cover

  - Maximization problem of vertex disjoint cycle cover
]

#slide[
  == Metaheuristics

  - Might be interesting for maximizing switches 
]

#slide[
  == Related Topics

  - Debt cancelling
  - Kidney exchange
  - School choise
]

#slide[
  == Two True one False

  - let $x =$ "I cycled from Bergen to Voss in under 6h"
  - let $y =$ "Fredrik was also my dads master advisor"
  - let $z =$ "I have reduced from 3-SAT to hamiltonian path"

  #uncover("1-")[
    $
    (z or z or z) and (y or y or y) and (not x or not x or not x)
    $
  ]
]
