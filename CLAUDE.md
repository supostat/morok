# CLAUDE.md

## Language
- Everything under `docs/` is written in English only.
- Code comments are written in English only.

## Comments
- Code documents itself: names, extracted predicates and small focused methods carry the meaning.
- Comments are rare and explain only what the code cannot express — the why, never the what.
- Comments never reference process artifacts: no phase numbers, task numbers, step labels or similar.

## Architecture first
- Design before writing: settle module boundaries, responsibilities, dependencies and data flow first.
- Code is architecturally sound from its first version; no "restructure later" shortcuts.

## Workflow
- Code is written through the mneme plugin workflow (`/mneme:*`).

<!-- MNEME-CRITERIA-CONTRACT -->
## mneme: phase criteria contract

- A done-when criterion is ONE argv command — no quotes, no `&&`/`|`: the gate-runner spawns a
  single process, and any quote or shell operator is an instant red `malformed-command` gate.
- Executable-first: a criterion is a command with exit 0 and a DEFINITE target (a specific test
  file, a `grep -q MARKER path`) — never a bare full-suite run alone. An agent-judged criterion
  is allowed only where the outcome is fundamentally not machine-checkable, and is marked
  `agent-judged` explicitly.
- Every criterion command must EXIST in the project (a script in package.json, a file on disk) —
  verify before writing the spec, never guess.
<!-- /MNEME-CRITERIA-CONTRACT -->
