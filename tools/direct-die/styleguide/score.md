# The score scale

This file states the score scale. It applies to every asset type.

The critique loader puts this text into every critique prompt, and the
review interface shows the score that comes back. This file is the only
place that states the scale. Change it here and nowhere else.

The score is a whole number from 0 to 100. It has fixed anchors, so a
score means the same thing in round 0 and in round 9. Two scores from
different rounds are comparable.

| Score | Meaning |
|-------|---------|
| 90 to 100 | Accept it. The asset is ready to ship. It breaks no rule, and it matches the accepted exemplars. |
| 75 to 89 | Nearly ready. It breaks no rule. One fault of taste remains. |
| 60 to 74 | Sound, but weak. It breaks no rule, and it reads at the display size, but it is dull or generic. |
| 40 to 59 | It breaks one rule, or it reads poorly at the display size. |
| 20 to 39 | It breaks two rules or more. |
| 0 to 19 | It is not the asset that was asked for, or it does not render. |

Judge the score against the anchors above. Do not judge it against the
score of an earlier round, and do not move the score to show progress. A
drawing that fixes one of two broken rules moves from the 20 band to the
40 band. It does not reach the 60 band until it breaks no rule.
