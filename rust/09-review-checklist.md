# 9. Review checklist

- [ ] Correct observable behavior, errors and public contracts?
- [ ] Ownership, borrowing and lifetimes deliberate rather than excessive cloning?
- [ ] Invariants represented and invalid states handled appropriately?
- [ ] Methods, free functions, traits, generics and enums chosen for their Rust semantics?
- [ ] Boundaries justified by real responsibility, not forced one-type-per-file rules?
- [ ] Selected architecture profiles implemented without unnecessary layers?
- [ ] Concurrency, cancellation, blocking and backpressure considered where relevant?
- [ ] Unsafe soundness and FFI contracts documented and checked?
- [ ] Relevant tests, target matrices and performance evidence present?
- [ ] API, README, compatibility and security implications addressed?

Review findings must state a concrete risk or conflict with an explicit project contract. Purely stylistic advice is not a universal blocker.
