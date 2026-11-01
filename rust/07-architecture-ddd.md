# 7. Architecture is a project decision

No single architecture fits libraries, backends, operating systems, games, embedded systems and scientific workloads. State invariants, ownership, performance and failure modes first; choose patterns afterward.

## Profiles

| Profile | Typical use | Principal tools |
|---|---|---|
| Simple or layered | CLI, CRUD, small applications | modules and functions |
| DDD | complex business language and invariants | bounded contexts, value objects, aggregates |
| Hexagonal | substitutable external systems | ports, adapters and composition |
| Clean architecture | policies independent of volatile details | dependency inversion |
| CQRS / event sourcing | distinct read models, audit or replay | commands, projections, versioned events |
| Actor / messaging | concurrent independent components | ownership transfer, bounded mailboxes, supervision |
| Systems / embedded | kernels, drivers, firmware | RAII, unsafe contracts, typestate |
| Performance / data-oriented | engines, ECS, math and analytics | slices, SoA, arenas, SIMD and benchmarks |
| Functional core | reproducible computations | pure transformations and explicit effects |
| Plugin | extensible programs | stable contracts, ABI boundaries and feature negotiation |

Profiles may be combined. No profile mandates a particular directory layout.

## DDD (optional)

Strategic DDD defines bounded contexts, ubiquitous language and context relationships. **A bounded context does not imply exactly one crate.** Multiple contexts may share a crate with explicit module boundaries, and one context can span crates.

Tactical DDD defines entities by identity, value objects by value, aggregates by consistency boundaries, domain services and events. Enforce relevant invariants, but do not apply aggregates to every data structure. One aggregate per transaction is not a universal rule: transaction boundaries follow domain consistency needs.

Repositories and domain events exist only when they solve persistence or integration needs. A domain operation may be a free function, struct method or service depending on state and responsibilities.

## Hexagonal and Clean (optional)

Ports can be traits, function parameters or message contracts. Introduce a port when an external dependency needs isolation, test substitution or stability—not as a mandatory wrapper for every file operation. Async I/O ports are valid; consider object safety and dispatch costs. Compose concrete adapters at a suitable boundary.

Avoid copying C# `IService -> Service -> IRepository -> Repository` layers without domain justification.

## Systems and high performance

Kernels, device drivers, graphics engines and embedded code emphasize memory ownership, deterministic lifetime, target constraints, safe unsafe-boundaries and layout. Data-oriented internals are not violations of DDD because DDD need not apply. Measure before imposing an abstraction on hot paths.

## Decisions

Document selected profiles, invariants, dependency contracts, alternatives and revision triggers. Boundaries can be module-level, crate-level, process-level or protocol-level according to actual requirements.
