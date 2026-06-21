//! `integration_tests` has no contract of its own.
//!
//! It exists purely to host cross-contract, end-to-end tests that exercise
//! `production_escrow` and `registry` together the way an off-chain indexer
//! would: calling escrow lifecycle methods, then mirroring those changes
//! into the registry's activity log, and asserting both contracts agree on
//! campaign state at every step.
//!
//! See `tests/campaign_lifecycle.rs` and the crate `README.md` for details
//! on running the suite.
