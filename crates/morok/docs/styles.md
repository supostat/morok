# Styles

An application is styled by CSS files that the build compiles into Rust tables
([An application](application.md#styles-of-its-own)): no CSS is read while the application runs.
The compiler takes a subset of CSS and refuses everything else, with a diagnostic at the place.

**Which properties, states and pseudo-elements the subset holds, and what each property takes, is
listed in `crates/css/docs/css-subset.md`.** That file is rendered from the tables of the compiler
itself. This chapter tells what no table holds: how the pieces fit together.

## Tokens

Every color, shadow, font and duration of a stylesheet is a **token**: a custom property declared
in the tokens file of the application. A stylesheet names a token with `var()` and never writes
such a value out.

The tokens file holds two blocks and nothing else: `:root { … }` with the values under the light
color scheme, and `@media (prefers-color-scheme: dark) { :root { … } }` with the values under the
dark one. Both declare the same names. `design/system/tokens.css` is the tokens file of the kit;
`crates/catalog/tokens/base.css` is that of an application, with tokens of its own added.

The family of a token is the start of its name, and decides what the token holds:

| Name | It holds | For example |
|---|---|---|
| `--color-…` | A color. | `--color-accent: #148F78` |
| `--space-…`, `--radius-…` | A length in px. | `--space-3: 12px` |
| `--font-…` | A list of font families. | `--font-ui: -apple-system, system-ui, sans-serif` |
| `--text-…` | A font size in px. | `--text-md: 13px` |
| `--text-leading…` | A line height without a unit. | `--text-leading: 1.45` |
| `--text-weight-…` | A font weight from 1 to 1000. | `--text-weight-semibold: 600` |
| `--shadow-…` | A list of shadow layers. | `--shadow-float: 0 8px 24px rgba(17, 19, 24, 0.14)` |
| `--motion-…` | A duration or an easing. | `--motion-fast: 120ms` |

A token is used where its kind fits: a length token wherever a length goes, a color token wherever
a color goes. The other way round is refused: `gap: var(--color-bg)` answers
`` `--color-bg` is a color, not a length ``.

In Rust a token is a field of the generated `TOKENS` table: `--color-accent` is
`TOKENS.color_accent`, a `Token<Color>`. `Theme::resolve` gives its value under a theme, which is
how a canvas draws in the colors of the tokens ([Pictures](pictures.md#canvases)).

## Classes and states

A rule styles the elements of a class. A class name is lowercase kebab-case, and becomes a field
in snake case of the generated `CLASSES` table: `.note-title` is `CLASSES.note_title`. An element
is given classes when it is built ([Elements](elements.md#classes)).

```css
.note {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  background-color: var(--color-surface);
  color: var(--color-text);
  font-family: var(--font-ui);
  font-size: var(--text-md);
  line-height: var(--text-leading);
}

.note:hover {
  background-color: var(--color-surface-2);
}

.note.pinned {
  font-weight: var(--text-weight-semibold);
}

.note:checked,
.note.pinned:hover {
  background-color: var(--color-selection);
}

.notes > .note {
  flex-shrink: 0;
}
```

The states are pseudo-classes. The tree sets `:hover`, `:active`, `:focus`, `:focus-visible` and
`:focus-within` itself; a component sets `:disabled` and `:checked` with
`ElementBuilder::state`. `:checked` is the state of whatever is on or chosen: a checked box, a
selected row, a pressed toggle.

## The shape of a selector

A selector is:

- one or more classes of one element, with any number of states: `.note.pinned:hover`;
- before it, at most one parent step with `>`: `.notes > .note`, where the parent too is classes
  and states;
- after it, at most one pseudo-element: `.field::placeholder`;
- several such selectors separated by commas.

Nothing else selects: no element names, no IDs, no attributes, no `*`, no descendant step written
with a space, no sibling steps, no `:nth-child()`, `:not()` or `:has()`. Where CSS would use one of
them, the element gets a class of its own.

A rule with more classes and states in its selector is over a rule with fewer; among equals the
later rule is over the earlier. The stylesheets of the kit come first, then those of the
application in the order of their paths. An inline declaration is over every rule.

Both are compiled into one style sheet, so a selector of an application may name a class of the
kit, as the catalog does to size its lists with `.page-row > .scroll-view` in
`crates/catalog/styles/pages.css`. The classes of the kit are declared in the stylesheets beside
the components, in `crates/components/src`.

## Pseudo-elements

There are two, both for edited text, and a rule of each sets one property.

```css
.search::placeholder {
  color: var(--color-text-3);
}

.search:focus::selection {
  background-color: var(--color-selection);
}
```

Anything else CSS would draw with a pseudo-element is an element of its own.

## Units, `var()` and `calc()`

- A length is in `px` or is a length token; a zero needs no unit. Where the property allows it, a
  percentage: `50%`. There is no `em`, `rem`, `vw` or `vh`.
- An angle is in `deg`. A time is in `ms` or `s`.
- `var()` takes the name of a token and nothing more: no fallback after a comma.
- `calc()` adds and subtracts lengths, percentages and tokens, and multiplies and divides them by
  numbers.

```css
.sheet {
  width: calc(100% - var(--space-6) * 2);
  max-width: 640px;
  min-height: 50%;
  margin: 0 auto;
  padding: calc(var(--space-4) + 2px);
  border: 1px solid var(--color-line);
  transform: rotate(2deg);
}
```

## Colors, gradients and shadows

A color is a color token, `transparent` or `currentColor`. A shadow is a shadow token or `none`. A
background is a color, one `linear-gradient()` or `radial-gradient()` whose stops are colors of
the same three kinds, or both.

```css
.card {
  background: linear-gradient(var(--color-surface), var(--color-surface-2));
  border: 1px solid var(--color-line);
  box-shadow: var(--shadow-float);
}
```

## Text

`font-family` takes a `--font-…` token, `font-size` a `--text-…` size token or a `calc()` over
one, `font-weight` a `--text-weight-…` token, `normal` or `bold`, and `line-height` a
`--text-leading…` token or `normal`. The color and the properties of text pass from an element to
its children, so a window sets them once, on its outermost element.

## Transitions and `@keyframes`

A `transition` names the properties that change smoothly: lengths, colors, `box-shadow`, `opacity`
and `transform`. Its duration and its easing are values or `--motion-…` tokens.

An `animation` names a `@keyframes` rule of any stylesheet of the application, declared before or
after it. A keyframe is `from`, `to` or a percentage, and sets the same properties a transition
changes. The name of the keyframes is one identifier.

```css
.badge {
  opacity: 0.6;
  transition:
    opacity var(--motion-fast) var(--motion-ease),
    background-color 200ms;
}

.badge:hover {
  opacity: 1;
}

.badge.fresh {
  animation: badge-arrive var(--motion-slow) var(--motion-ease-out) both;
}

@keyframes badge-arrive {
  from {
    opacity: 0;
    transform: translate(0, 8px) scale(0.96);
  }

  60% {
    opacity: 1;
  }

  to {
    transform: translate(0, 0) scale(1);
  }
}
```

While the system asks for less motion, a transition ends at once and an animation that never
ends does not run.

## What the compiler refuses, and what it says

A diagnostic names the file, the line and the column, shows the line and gives a hint:

```text
error[morok-css]: descendant selectors are not supported
  --> components/list.css:14:1
   |
14 | .list .row .title { font-weight: 600 }
   | ^^^^^^^^^^^^^^^^^ give the element its own class: .list-title
```

The build prints every diagnostic of every stylesheet, in the order of the files and the lines,
and fails.

| What is written | What the compiler says |
|---|---|
| `.list .row { … }` | descendant selectors are not supported |
| `.a > .b > .c { … }` | only one child step is supported |
| `div { … }` | type selectors are not supported: `div` |
| `#main { … }` | ID selectors are not supported |
| `.a + .b { … }` | sibling combinators are not supported |
| `* { … }` | the universal selector `*` is not supported |
| `.row[data-kind] { … }` | attribute selectors are not supported |
| `.row:nth-child(2) { … }` | `:nth-child()` is not supported |
| `.a:visited { … }` | the pseudo-class `:visited` is not supported |
| `.a::before { … }` | the pseudo-element `::before` is not supported |
| `:hover { … }` | a state needs a class to belong to |
| `.Button { … }` | `.Button` is not a lowercase kebab-case class name |
| `.self { … }` | `.self` cannot name a class |
| `color: #fff` | raw colors are not supported |
| `box-shadow: 0 1px 2px var(--color-line)` | raw shadows are not supported |
| `color: var(--color-nope)` | unknown token `--color-nope` |
| `color: var(--space-2)` | `--space-2` is a length, not a color |
| `gap: var(--space-2, 4px)` | a `var()` fallback is not supported |
| `float: left` | `float` is not a supported property |
| `display: inline-flex` | `display` does not support this value |
| `padding: 1em` | the unit `em` is not supported |
| `border-radius: 50%` | a percentage is not supported here |
| `background-image: url(pattern.png)` | `url()` is not supported |
| `color: var(--color-text) !important` | `!important` is not supported |
| `@media (min-width: 100px) { … }` | `@media` is not supported |
| `--own-gap: 4px` in a stylesheet | `--own-gap` is declared in a component stylesheet |
| `animation: missing 1s` | no `@keyframes` rule is named `missing` |
| `transition: display 1s` | `display` cannot be transitioned |

When a value is refused, the hint says what the property takes, in the words of the reference:
`display: inline-flex` answers ``it takes `block`, `flex`, `grid` or `none` ``.

Three more facts follow from the list:

- There is no `@media`: a window does not restyle itself by its width. The color scheme is the
  business of the tokens file alone.
- There is no `@import` and no `@font-face`: every `.css` file of the stylesheets directory is
  compiled, and fonts are those of the system, named by the `--font-…` tokens.
- `border-style` is `solid` or `none`, and `position` is `relative`, `absolute` or `sticky`.

The tests that pin these refusals are `crates/css/tests/subset_errors.rs`.
