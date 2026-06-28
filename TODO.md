# TODO - router-execution backoff retry tests

- [ ] Inspect existing backoff/retry tests in `contracts/router-execution/src/lib.rs`.
- [ ] Add missing delay-calculation test cases for `compute_backoff_ms`:
  - [ ] base_ms=100, multiplier=200 with attempt mapping to execute attempts
  - [ ] exponential growth across attempts 1-5
  - [ ] multiplier=300 large multiplier sanity (no overflow; monotonic)
  - [ ] multiplier=100 constant delay boundary
  - [ ] base_ms=0 behavior across attempts
- [ ] Add/extend tests for max retries exhaustion behavior in `execute()`.
- [ ] Run `cargo test` for the affected crate/workspace.
- [ ] Update TODO checklist as tests pass.

