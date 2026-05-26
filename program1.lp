% choice: genau eins von a oder b
1 { a; b } 1.

% choice: optional c oder d (mindestens eins)
1 { c; d }.

% e ist immer wahr
e.

% kleine Abhängigkeiten für Variation
d :- b.
c :- a.

% optional: Konflikte (kannst du an/aus machen)
% :- a, b.