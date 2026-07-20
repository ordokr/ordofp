**What this changes and why**

**Verification** (there is no hosted CI — a green local gate is the merge bar)

- [ ] `cargo run -p xtask -- all` passes locally
- [ ] `cargo run -p xtask -- deep` (Miri + fuzz smoke) if `unsafe` was touched
- [ ] New public APIs have doc comments with a runnable example
