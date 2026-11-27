# Design rules of the pages of MOROK

The shared layer of the pages designed under `design/pages`: what they link, what they may not
do, and how a choice between two ways is made. The visual language of the kit itself — its
palette, type and components — is `design/morok-design-book.html`; this file is about pages that
present the kit.

## The shared layer

- `tokens.css` — the tokens of the kit, light and dark by the color scheme of the system. The
  compiler of the kit reads this file: a page never adds a token to it.
- `tokens-dark.css` — the dark values alone, for a page that is dark whatever the system says.
  It mirrors the dark block of `tokens.css` and holds nothing of its own. No hand writes it:
  `cargo xtask site-tokens .` renders it from the tokens, and `cargo xtask check` fails while it
  differs from what they render.
- `components.html` — the patterns pages share, once a pattern has appeared on two of them.

A page links the layer and never copies a value out of it: every color, size and duration in a
page is a `var(--…)` or a multiple of one.

## Selection rules

- **A picture of the kit is a frame of the kit.** A window or a component on a page is a picture
  the kit rendered, never a drawing of one. A hand-drawn stand-in belongs to a draft.
- **Dark by design is a decision of the page.** A page either follows the system (it links
  `tokens.css` alone) or is dark always (it links `tokens-dark.css` after it). It does not switch
  by hand.
- **Motion shows what the kit does, or is not there.** A window that arrives, a rule that draws a
  control, a list that filters: these say something. Motion that only decorates is left out.
- **Durations are the motion tokens and their multiples.** No duration is written as a number of
  its own.
- **Every animation has a still form.** With reduced motion, and in a browser without scroll
  timelines, the page stands in its final state and loses nothing.
- **One primary action a screen.** The guide is the primary action of every page about the kit;
  the repository is the second.
- **A page states what the code holds.** A number, a requirement or a claim on a page has a place
  in the repository it was read from.

## Anti-patterns

Named so that a review can point at them. No detector exists for these: they are for people.

- nested-cards — a card inside a card reads as chrome, and the content loses its edge.
- gradient-text — it decorates the words in place of what they say.
- ai-color-palette — purple and teal on a dark field is the look of a generated page.
- pulsing-dot — it claims liveness nothing behind it has.
- marketing-buzzword — "streamline" and "empower" say nothing a reader can check.
- low-contrast — a style that costs the reading is paid for by the reader.
- tight-leading — dense lines tire the eye before the paragraph ends.
- justified-text — on the web it opens rivers of white through the text.
- all-caps-body — body copy in capitals shouts and reads slower.
- oversized-h1 — a heading that pushes the content off the first screen.
- dark-glow — a halo around every panel; one light for the scene is the limit.
- marquee — text that runs by steals attention from the text that stays.
- icon-tile-stack — rows of icon tiles fill a section and tell nothing.
- cream-palette — the safe beige of a page nobody chose a color for.
- flat-type-hierarchy — every line of one weight, so nothing leads.

## What the checks do not see

The check of raw values in an etalon is a heuristic: it finds a hex color and a `px` literal in a
declaration without a `var(--…)`. It does not find a raw value that sits beside a token in one
declaration, a duration written out, or a value in a stylesheet the etalon links. Those are held
by review.
