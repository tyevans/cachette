# The Reader Rubric (Index)

This document holds the questions that a fresh reader answers about a
documentation page, and the scale that grades the answers.

**A fresh reader has not read the source and has not read this repository.**
That reader is the audience the product record names.[^1] A reader who has read
the source cannot grade the page, because that reader supplies from memory what
the page does not say.

## How to use this

Give a reader the address of one page and nothing else. Give the reader the
questions below. Do not give the reader the source, the records, or this
document's own history.

Run the questions again after each repair. A score that does not move says the
repair missed the reader.

## The questions

### A. Orientation

1. What does this library do? Answer in one sentence, from this page alone.
2. Who is it for, and what would somebody build with it?
3. What is the unit of work? Name the thing a caller acts on.

### B. First action

4. Could you write a program that runs? Write the first five lines.
5. What must happen before the first call? Name every step you must guess.
6. Where would you look next, and does this page tell you?

### C. Vocabulary

7. Which terms does the page use without defining them?
8. Which term did you assume a meaning for? State the meaning you assumed.
9. Does one word carry two meanings anywhere on the page?

### D. Values

10. For each value a call returns, do you know its type?
11. Do you know the unit of every number? Name any number whose unit you
    cannot state.
12. What would you get wrong if you trusted your first reading of a number?

### E. Failure

13. What can fail, and what would you catch?
14. Which error would you not know how to prevent?
15. Does the page say what a call does when it refuses?

### F. Gaps

16. What did you expect to find and not find?
17. Which question did you have to leave unanswered?
18. What would you have to read the source for?

### G. Trust

19. Does anything on the page look wrong or contradict itself?
20. Which statement would you not act on without checking?

### H. Interface

21. Does the shape of the interface match the job? Name any call that is
    awkward to use correctly.
22. What did you go looking for and not find? Name the call you expected.
23. What does the interface expose that a caller should not have to think
    about?
24. Which name would you change, and what would you call it?
25. What state or ordering does the interface make you hold, that it could
    hold for you?

**Say which repair each complaint needs.** A complaint that the page is
unclear asks for prose. A complaint that the interface is wrong asks for a
different interface. The project routes the two differently, and a reader who
mixes them sends the work to the wrong place.

## The scale

Grade each dimension from 1 to 5. State the evidence for the grade, and name
the one repair that would raise it by one point.

| Grade | Meaning |
|---|---|
| 1 | The reader cannot proceed. The page does not answer the question at all. |
| 2 | The reader guesses. The page gives a fragment, and the guess is unsafe. |
| 3 | The reader proceeds and is wrong some of the time. |
| 4 | The reader proceeds correctly, after re-reading or inference. |
| 5 | The reader proceeds correctly on the first reading. |

Grade these eight dimensions: orientation, first action, vocabulary, values,
failure, gaps, trust, interface.

**Report the lowest grade first.** The lowest grade is the next repair.

## What a grade must carry

A grade with no quotation is an opinion. Quote the sentence that earned the
grade, or state that no sentence on the page addresses the question.

## References

[^1]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
