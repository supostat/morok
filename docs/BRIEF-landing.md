# Brief: the landing page of MOROK

A brief for the design of the landing page. It holds what is decided, the facts the page may
state, and the references looked at on Mobbin. It decides nothing about the look: the directions
at the end are proposals for the design to weigh.

## What is decided

- A landing page comes first, before the rest of the documentation for people.
- It is hosted on GitHub Pages: static files.
- Its reader is a developer who is choosing what to write a macOS application in.
- On the page the reader learns what the kit is and what it is not, sees what its components look
  like, and goes on to the guide and to the repository.

## Not decided

- The language of the page.
- Whether the page opens light or dark, and whether it switches.
- Whether its pictures are frames the kit renders itself or drawn mock-ups.

## Facts the page may state

Each is in the repository; the page states nothing the code does not hold.

- MOROK is a UI kit for macOS that draws the whole interface itself through Metal: no `AppKit`
  controls and no browser (`README.md`).
- An application styles its windows with a subset of CSS compiled at build time, and changes them
  by writing signals (`crates/morok/docs/overview.md`).
- The name: morok is a glamour, an illusion — the interface looks like CSS from a browser, yet
  there is no browser (`docs/MOROK-ARCHITECTURE.md`).
- It needs macOS 26 or later on Apple Silicon and Rust 1.94 or later (`README.md`).
- The components come in ten families, each on a page of the catalog (`crates/catalog/src/pages`),
  with two whole screens built from them: a file manager and a disk map.
- The visual language of the kit — its colors, type and spacing — is in
  `design/morok-design-book.html`, and its tokens in `design/system/tokens.css`.
- Where the page leads: the guide, `crates/morok/docs/overview.md`; the reference of the CSS
  subset, `crates/css/docs/css-subset.md`; the repository, `https://github.com/supostat/morok`.

## References from Mobbin

Website sections of developer products, looked at on 2026-11-26. What each shows is described
from its picture.

### Hero

- [Grammarly for developers](https://mobbin.com/sites/sections/d6cbb462-28bd-4e04-8338-e8d7785c04c0?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — the statement and one button on the left; on the right the product at work, with the few lines
  of code that produce it floating over its corner. The code and its result are one picture.
- [Miro Developers](https://mobbin.com/sites/sections/ccb3ed6f-ea96-4486-9578-8be87aa19f55?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a dark split hero with two buttons, an illustration and a code card that leans in from the
  edge; the next section begins in a light color under it.
- [Frontify developer platform](https://mobbin.com/sites/sections/67380933-75aa-40ed-b13c-617db6250c9c?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a dark split hero; the right half is a laptop showing a terminal beside the interface it built.
- [1Password Developer](https://mobbin.com/sites/sections/330752c2-0212-49ba-a09a-ea977cbab9b7?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a centered statement, a paragraph, two buttons side by side (docs first), a wide picture under
  them.
- [Canva Developers](https://mobbin.com/sites/sections/69ba4b70-608e-44f8-bbcb-c75d1465e6dd?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a centered statement in which a few words are drawn as a selected frame of the product's own
  editor: the headline shows the product's signature gesture.
- [Shopify Editions](https://mobbin.com/sites/sections/1a1c2145-71da-40a3-9e78-56b515a907e8?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a very large condensed word over a centered statement and a glowing frame of the product.

### Showing components

- [Framer Motion](https://mobbin.com/sites/sections/dc938a66-a9a8-48e7-87bc-8b925e61d1b4?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — one card in two halves: a live demo on a colored field on the left, the code that makes it on
  the right. The strongest pattern here for a kit of components.
- [Webflow, a component library](https://mobbin.com/sites/sections/9065ff94-e193-40e4-9898-48f25dd16b77?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a sidebar that lists the families of components, and beside it the previews of the chosen
  family, each under a small tab with its name. It is the shape of the catalog of the kit itself.
- [Webflow libraries](https://mobbin.com/sites/sections/fd4a150f-5e30-4d17-af7d-af0b2a4c09f4?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a grid of cards, each a picture with a count of what it holds.

### Features and code

- [Stripe, developer-centric](https://mobbin.com/sites/sections/3f00c318-d158-43a9-aee5-d0e69166e1a7?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a small colored label, a heading, a paragraph and a checklist on the left; a code panel with
  tabs on the right.
- [Stripe, designed for developers](https://mobbin.com/sites/sections/286613b2-cec3-4b45-ba6e-77930aa42ef4?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — the code on top of the terminal output it gives; under both, four short columns, each an
  icon, a line and a link.
- [Retool, how it works](https://mobbin.com/sites/sections/cc463b41-675a-4256-ac9f-88ce9f519687?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — numbered steps as a list on the left, of which one is open; the product on the right changes
  with the step.
- [Bird, developer ecosystem](https://mobbin.com/sites/sections/cddb70d8-8531-4a78-991e-d472aeeb2d95?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a code panel with tabs centered on a tinted field, three plain columns of text under it.

### Getting started

- [Giga](https://mobbin.com/sites/sections/a2004615-5fee-4e7d-9f84-602f3f1ab1f0?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — one line: the install command in a pill with a copy button, one button beside it, the license
  named in the sentence above.
- [Height](https://mobbin.com/sites/sections/89698f71-3913-41e3-8ef6-72656eb04453?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — two ways in side by side, and under them eight short facts in a grid, each an icon, a line in
  bold and two lines of text.
- [Graphical](https://mobbin.com/sites/sections/1ac17256-7c1f-48f2-a732-7c9a6a5ba395?utm_source=claude-code&utm_medium=ai-assistant&utm_source_platform=mcp)
  — a drawn terminal window with the command typed in it.

## Three directions

- **A. The code beside what it draws.** A split hero: the statement on one side, a real window of
  the kit on the other with the CSS that styles it floating over its corner (Grammarly). Below,
  cards of a component beside its code (Framer Motion, Stripe). It puts the one unusual claim of
  the kit — CSS without a browser — on the first screen.
- **B. A centered showcase.** A centered statement, two buttons, a wide frame of the catalog
  (1Password, Canva, Shopify Editions); then facts in a grid and the install line (Height, Giga).
  The familiar shape of a product page; it says less about what is particular to the kit.
- **C. The catalog as the page.** A short hero, then the families of components in a sidebar with
  their previews (Webflow), as the catalog application shows them. It shows the most components,
  and repeats what the catalog and the guide already do.

Proposed: A, with the getting-started line of B.
