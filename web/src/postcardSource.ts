// Paste the final validated content of postcard.jianpu verbatim between the backticks:
export const POSTCARD_SOURCE = `\
[metadata]
title = "Jianpu Postcard"
author = "—"
subtitle = "Complete syntax reference — every feature, one measure each"
max columns = 17
row height = 24
label width = 120
note number width = 8

[parts]
Chords (C) = chord
Melody (M) = notes lyrics

[score]
label="Scale degrees 1–7 & rest 0"
bpm=120 key=C4 time=4/4
1 - - -
1 2 3 0
do re mi _

label="Octave: up (') & down (,)"
"
1' 2' 1, 2,
_

label="Duration: eighth (_) & sixteenth (=)"
"
1_ 1_ 1= 1= 1= 1= 1 -
_

label="Duration: dotted (.)"
"
1. 2_ 1. 2_
_

label="Duration: extension (-)"
5 - - -
1 - - -
do - - -

label="Slur group"
"
(1 2) (3 4)
_

label="Nested slur"
"
((1_ 2_) 3) 4
_

label="Cross-measure slur (bar 1 of 2)"
"
1 2 3 (4
_

"
5) 6 7 0
_

label="Chord: major & minor (m)"
1 1m 1 1m
1 - - -
_

label="Chord: diminished (o) & augmented (+)"
1o 1+ 1o 1+
1 - - -
_

label="Chord: dominant 7th (17)"
17 4m7 57 1
1 - - -
_

label="Chord: major 7th (1M7) & minor 7th (1m7)"
1M7 4m7 1m7 1
1 - - -
_

label="Chord: slash bass (1/5)"
1/5 - 5/7 -
1 - 5 -
_

label="Chord: rest (0) & extension (-)"
0 - 1 -
1 - 5 -
_

label="Directive: bpm, key, time"
bpm=96 key=G4 time=3/4
"
1 2 3
_

label="Inline time change (N/D)"
bpm=120 key=C4
"
4/4 1 - - -
_

label="Inline key change (1=)"
"
1=D4 1 2 3
_

label="Inline BPM (bpm=)"
key=C4
"
bpm=72 1 - - -
_

label="Lyrics: CJK syllables"
bpm=120 time=4/4
1 - - -
1 2 3 4
春 天 来 了

label="Lyrics: Latin syllables"
"
1 2 3 4
do re mi fa

label="Lyrics: syllable break (-)"
"
1_ 2_ 3 4
hel- lo world !

label="Lyrics: held syllable (-)"
"
1 - 2 -
spring - here -

label="Lyrics: no-lyrics marker (_)"
"
1 2 3 4
_

label="Ditto"
1 2 3 4
"
"
`
