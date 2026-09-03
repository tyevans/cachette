# Documentation Rules

These rules apply to all prose in this project. Prose means documentation,
ADRs, README files, code comments, docstrings, commit messages, and any
text a reader sees.

These rules do not apply to code, identifiers, data, or command output.

## 1. Simplified Technical English

All prose must conform to Simplified Technical English (ASD-STE100).

Apply these rules:

- Write short sentences. Use 20 words or fewer for descriptive text.
  Use 20 words or fewer for procedural steps.
- Write one idea in each sentence.
- Use the active voice. Name the agent that does the action.
- Use one word for one meaning. Do not use synonyms for variety.
- Use the approved meaning of a word. Do not use a noun as a verb.
- Write instructions as commands. Start the sentence with the verb.
- Use simple tenses. Prefer the present tense.
- Do not omit articles. Write "the buffer", not "buffer".
- Do not use a noun cluster of more than three words.
- Do not use rhetorical questions, metaphors, humour, or emphasis
  for effect.

Write paragraphs of six sentences or fewer.

## 2. Documents must stand alone

A reader must understand a document without opening another document.

- State the context that the document needs. Do not assume the reader
  read a different document first.
- Define each term at its first use, or link the term to the project
  glossary through a footnote.
- Repeat necessary information. Repetition is correct when it keeps a
  document self-contained.
- Do not write "see the other document for details" in the body text.
  Include the details, or state the conclusion and cite the source in a
  footnote.

A document that only points at other documents is an index. Mark it as
an index in its title.

## 3. References go in footnotes only

Body text must not contain a reference to external material.

External material means another document, a file path, a URL, an issue
number, a specification, or a third-party source.

Use a numbered footnote marker in the body text. Put every footnote
definition in one section at the end of the document.

### Format

Use Markdown footnote syntax. Put the marker directly after the claim
that it supports.

```markdown
The pyramid holds integer accumulators only.[^1] Float addition is not
associative, so a float sum is not a monoid.[^2]

...

## References

[^1]: ADR-0002, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: IEEE 754-2019, clause 5. https://ieeexplore.ieee.org/document/8766229
```

### Rules for footnotes

- Put the `## References` section last in the document. Use this exact
  heading.
- Number the footnotes in the order that they occur in the body.
- Write one line for each footnote definition.
- Give the source name first. Give the location second. Give the URL or
  path last.
- Do not put a URL, a file path, or a document name in the body text.
- Do not use an inline Markdown link in the body text.
- Do not repeat a footnote. Reuse the same marker if the same source
  supports more than one claim.

### Why this format

One section holds every external reference. A path or a URL changes in
one place. A reader checks the reference list to find each dependency of
the document.

## 4. Exceptions

These rules do not apply to:

- Code blocks, and the identifiers or file paths inside them.
- Tables that list files, paths, or versions as data.
- Generated output, logs, and error messages that the document quotes.
- The `## References` section itself.

A cross-reference inside one document is not external material. Write it
as a normal Markdown heading link.
