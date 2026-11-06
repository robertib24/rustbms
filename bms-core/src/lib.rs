//! Battery management logic: everything the ECU runs.
//!
//! `no_std` outside unit tests, so the same code builds for the host and for a
//! Cortex-M4F (`thumbv7em-none-eabihf`). Math that `core` lacks, such as `exp`,
//! comes from `libm`.
//!
//! Sign convention, everywhere: current `i > 0` is discharge.
#![cfg_attr(not(test), no_std)]

pub mod cell;
