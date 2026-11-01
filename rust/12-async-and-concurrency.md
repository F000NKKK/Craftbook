# 12. Async, threads, actors and cancellation

Choose a concurrency model based on workload, not on fashion. Async is useful for many concurrent waits; dedicated threads are suitable for blocking work and isolated long-running computation. Parallel computation can use scoped threads or dedicated parallel runtimes.

## Async contract

An `async fn` returns a future; no execution is guaranteed merely by constructing it. An executor polls it. Futures may be cancelled by dropping them, including during partial I/O, transactions or lock acquisition. Design cancellation-safe operations explicitly.

`Send` is needed for futures transferred between threads; it is not mandatory for every future or every domain type. Distinguish synchronization (`Sync`) from transferable ownership (`Send`).

Avoid blocking OS calls within high-concurrency async workers; choose asynchronous primitives or a bounded blocking pool where appropriate. Holding a synchronous mutex guard across `.await` can deadlock or stall; minimize lock scopes, and review any async-aware lock separately.

## Channels and actors

Use bounded queues by default where untrusted or bursty producers can overrun memory. Define what happens at capacity: wait, reject, drop, coalesce or backpressure. Actors should own their internal state, communicate with typed messages and expose supervision or restart semantics only where needed.

## Structured lifetimes

Track background tasks. Define shutdown, cancellation, error propagation and task ownership. Detached tasks without lifecycle control create observability and resource leaks.

## Time

Use monotonic clocks for elapsed time and deadlines. Wall-clock time is for timestamps. Timeouts require a defined cleanup policy; a timeout that abandons work is not necessarily a rollback.

## Testing

Test shutdown races, bounded capacity, cancellation between steps, timeout boundaries, error propagation and fairness where relevant. Use deterministic scheduling or model checking for tricky interleavings; normal unit tests cannot exhaust thread schedules.
