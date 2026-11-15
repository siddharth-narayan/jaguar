#import "@preview/finite:0.5.1": automaton, layout


#automaton(
  (
    q0:       (q0: "*"),
    q1:       (q1: "*"),
    q2:       (q2: "*", q3: "\\n"),
    q3:       (q4: "* [0-9](mult)"),
    q4:       (q5: "* [.](negate)"),
    q5:       (q6: "[0-9](mult)"),
    q6:       (q9: "-", q7: "* (mult)"),
    q7:       (q8: "* (negate)", q9: "-"),
    q8:       (q9: "* (negate)"),
    q9:       (q3: "\\n", q9: "*"),
  ),
  initial: "q2",
  final: ("q0",),
  layout: layout.custom.with(positions: (
    q0: (0, 0),
    q1: (2, 0),
    q2: (4, 0),
    q3: (6, 0),
    q4: (9, 0),
    q5: (12, 0),
    q6: (12, -2),
    q7: (12, -4),
    q8: (12, -6),
    q9: (12, -10)
  )),
  style: (
    transition: (curve: 1, label: (angle: 0deg,)),
    q6-q9: (curve: 3)
  ),
)