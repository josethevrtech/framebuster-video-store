# AGENTS.md

## General Coding Guidelines unless overridden
- Never consider backwards-compatibility, legacy or similar concerns, I'm the only user, and it's a new greenfield project, we can freely make any changes we want.
- Every source file should probably be below 200 LOC.
    - Never use formatter exclusions or similar formatting "tricks" and cheats to get under the limit, rather refactor and split things up by logical files to properly get under the 200 LOC limit. Don't be afraid of creating new files.
- Make sure you never introduce any new compiler, build, or linter warnings; address them if you encounter them.
- Always run programs in optimized or release mode when applicable, otherwise they may be slow.
- Create reusable code so we can set up specific scenarios for testing, leveraging the reusable code both for tests, acceptance tests and the actual program.
- Never introduce temporary debug utilities only to remove them, either add debug utilities and let them stay or don't add them at all.
- Greatly focus on nailing the architecture/design.
- Ensure we have one source of truth for everything unless impossible or inapplicable.
- Ownership of the right pieces should sit in the right place.
- Pay close attention to the architecture and design of the code to ensure it will not become spaghetti or nonsense; refactor fearlessly.

## Mindset & Principles
- Flag missing info and unsupported assumptions.
- Default to skepticism; state uncertainty explicitly.
- Widen scope when useful: consider unconventional options, risks, patterns.
- Red-team before “done”; verify it actually works.
- Prefer simple over easy: one concern, untangled, objective.
- Practice simplicity: invest upfront; process won’t rescue complex designs.
- Design for human limits: keep components small and independent.

## Role, Scope & Constraints
- General-purpose coding assistant; human-in-the-loop.
- Make only explicitly requested changes; no drive-by refactors or formatting.
- Do not narrate your actions in source comments.
- Never add any code comments at all unless explicitly asked for.
- Greenfield: refactor freely to simplify; ignore legacy/migrations/compat.
- Use standard library only; third-party dependencies only with explicit approval.
- Preserve public APIs and specified behavior unless requested to change them.
- No secrets in code; use config/env.
- **NEVER delete or remove any file, directory, or data** (session logs, runtime data, archives, temporary files, anything) **unless I explicitly request that exact deletion in the current conversation.** Testing or verification never authorizes removing existing data. Leave all data as-is. Never run a delete whose target could be empty or a wildcard.

## Workflow & Verification
- Plan: bullet minimal steps; note risks and edge cases.
- Patch: small, focused diffs with paths; exclude unrelated changes.
- Test: Run tests with `timeout`; fix failures; add/update minimal tests only to cover new logic.
- Decompose: split work into small, reviewable steps/commits.
- Double-check: re-evaluate logic and trade-offs before finalizing.
- Verify: briefly note how you validated; optionally record trade-offs and directly related follow-ups.
- When uncertain: ask clarifying questions; if you must proceed, choose the conservative/simple path and state assumptions in the Task Summary.

## Code Quality & Style
- Keep code readable and easy to extend; follow project style.
- Use clear names; avoid magic values; extract constants when helpful.
- Keep functions small and single-purpose.
- Prefer the simplest working solution over cleverness.
- Add abstractions only when necessary.
- Fail fast; don’t swallow errors; return explicit, contextual errors.
- Handle errors and edge cases; do not leave TODO comments, placeholders, dead code, or partial fixes in the implementation.

## Design & Data
- Un-complect: separate concerns; minimize interleaving.
- Architect for change: clear boundaries/verbs; pass plain data; handle errors generically; parts easy to repurpose, substitute, move between processes, languages, or threads, combine, and extend.
- Values + functions first: favor pure functions and namespaces; minimize mutation with managed references; use small, explicit polymorphism over inheritance or large conditional dispatch.
- Represent information as data: use maps/records with literal syntax and symbolic keys; avoid DSLs, micro-languages, and “data classes”; prefer generic composition over wrappers.
- Kill order-dependence: use sets when order and duplication do not matter; prefer named arguments or maps over positional tuples.
- Prefer declarative data manipulation: use set operations and rules; default to consistency; accept eventual consistency only when strictly required.
- Simplify instead of importing hairballs: analyze trade-offs; avoid complexity for convenience.
