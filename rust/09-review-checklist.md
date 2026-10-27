# 9. Review checklist

What a reviewer checks by hand, after CI is green. Everything the tools
check is not repeated here. Each item links to its rule.

## Architecture

- [ ] The change sits in the right layer and context; nothing points outwards
      ([7.3](07-architecture-ddd.md#73-layers)).
- [ ] New I/O goes through a port declared by the domain or application,
      implemented by an adapter ([7.4](07-architecture-ddd.md#74-tactical-building-blocks-in-rust)).
- [ ] Business rules live in the domain, not in use cases, adapters or UI glue.
- [ ] State of an aggregate changes only through its root's methods; other
      aggregates are referred to by id.
- [ ] No new manager, hub, global or field-less namespace struct
      ([7.7](07-architecture-ddd.md#77-no-managers-no-hubs)).
- [ ] Logic that belongs to another crate is reused from it, not copied.
- [ ] New architectural decisions have an ADR.

## Design of the code

- [ ] Method vs. free function vs. trait follows the decision order
      ([2.3](02-functions.md#23-method-associated-function-free-function-or-trait)).
- [ ] Each function does one thing at one level of abstraction; no flag
      parameters; commands and queries are separate ([2.2](02-functions.md#22-shape-of-a-function)).
- [ ] New traits have two implementations or a generic consumer; they are small.
- [ ] Values that carry meaning are typed (newtypes, enums, value objects);
      invariants are checked once, at construction ([4.1](04-types-and-errors.md#41-types-carry-meaning)).
- [ ] Errors are typed, say what failed with what input, are not swallowed,
      and are logged once ([4.4](04-types-and-errors.md#44-errors)).
- [ ] `expect` messages state the invariant; no `unwrap` outside tests.
- [ ] `async` only on genuine I/O ([4.8](04-types-and-errors.md#48-async)).

## Names and files

- [ ] Names are the glossary's words; new terms are added to the glossary
      ([7.1](07-architecture-ddd.md#71-ubiquitous-language)).
- [ ] No abbreviations, noise words or stutter ([3.2](03-naming.md#32-choosing-words)).
- [ ] Each file has one primary item named by the file; split by
      responsibility, not by kind ([1.2](01-files-and-modules.md#12-one-primary-item-per-file)).

## Tests

- [ ] New behaviour is tested at the lowest level that can test it
      ([8.2](08-testing-and-quality.md#82-test-pyramid)).
- [ ] Tests assert outcomes, are deterministic, have behaviour names
      ([8.3](08-testing-and-quality.md#83-how-a-test-is-written)).
- [ ] A bug fix has a regression test that failed before the fix.
- [ ] New port → fake + contract suite; new adapter → runs the contract
      ([8.4](08-testing-and-quality.md#84-ports-fakes-and-contract-tests)).

## Docs

- [ ] Public items have docs with `# Errors` / `# Panics` / examples as needed.
- [ ] Comments explain why; no history, no commented-out code.
- [ ] Docs, glossary and context map changed in the same PR as the code
      that made them wrong.
