# 14. Cargo, feature flags and compatibility

## Workspace design

Use a workspace when packages share development and dependency management. A new crate should have a justified API, testing, target, compilation or release boundary—not merely satisfy an architectural layer diagram.

## Feature flags

Prefer additive features. Cargo unifies features across dependency paths, so independently correct combinations may interact. Test the feature matrix your project supports. For mutually exclusive implementations, explicit compile-time conflicts and documented alternative builds are preferable to silently choosing one.

Support `no_std` and `alloc` intentionally; disable default dependency features where needed and test intended targets. Conditional compilation can hide broken targets from local CI.

## Public API and semver

Public trait changes can break implementers; variants can break exhaustive matches; adding public struct fields can break struct literals. Document whether an API is stable, experimental or internal. Consider `#[non_exhaustive]`, private fields and sealed traits where future extension is anticipated.

Reproducibility needs pinned toolchains, stable dependency resolution and appropriate Cargo.lock policy. Libraries published to crates.io still need compatibility testing against supported dependency ranges; binaries often commit lockfiles for reproducible builds.

## Supply chain

Audit dependency licenses, advisories, maintenance, MSRV (minimum supported Rust version) and supported targets. Licenses must reflect project obligations rather than a universal whitelist.

## Verification

At minimum, test declared targets, promised compiler versions, feature combinations and doctests for documented examples. Add build reproducibility and release checks where they materially affect users.
