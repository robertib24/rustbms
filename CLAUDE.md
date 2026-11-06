# CLAUDE.md

Two BMSs in Rust, compared A/B on one simulated pack modeled on published first-generation Taycan figures (198s2p, 33 modules of 6s2p, LG E66A 64.6 Ah cells): a baseline with conventional, foxBMS-style algorithms and an improved, model-based one. The plan, the numbers and every milestone check are in `docs/PLAN.md`; read the plan's section for the milestone you are on before writing code.

Progress is tracked by the checkboxes in `docs/PLAN.md`: tick a check in the same commit as the test that proves it.

## Crates

- `bms-core` (lib): everything the ECU runs, `#![cfg_attr(not(test), no_std)]`. Dependencies: nalgebra (`default-features = false`, `features = ["libm"]`), libm, and heapless from M3. Baseline and improved are two implementations of the same traits (`SocEstimator` and friends), chosen per run.
- `bms-cli` (bin): headless scenarios to CSV, later the comparison report.
- Still to add, each at its milestone, with `cargo new` from the repo root (it adds the crate to `members`): `bms-sim` (M1: pack plant, sensors, profiles; rand 0.10), `bms-can` (M5: dbc-codegen in build.rs, a Bus trait, SocketCAN and in-memory buses), `bms-app` (M5: eframe 0.37 dashboard).

## Rules

- No std in bms-core: `f64::exp` and `f64::powi` don't exist there, so use `libm`; no `Vec`, `String`, rand, file I/O or egui. Use fixed-size arrays, const generics and heapless instead.
- Current `i > 0` is discharge everywhere: model, EKF, protection limits and the DBC's signed PackCurrent.
- SI units in code (V, A, s, Ω, °C, capacity in Ah); SoC as a fraction from 0 to 1, percent only on CAN and in reports.
- Baseline and improved see the same inputs, sensor noise, offsets and seeds; each experiment changes one thing.
- The expected values in the plan's checks tables come from a prototype of the exact model and are the test assertions (`approx::assert_relative_eq!`). Never loosen or change one to get a test passing; if a value looks wrong, stop and say so.
- Run scenarios with `--release`.
- dbc-codegen setters truncate (−12.4 A goes out as −12.3 A): round yourself and use the `*_raw_val` setters.
- SocketCAN is Linux only; keep it behind the Bus trait.

## Commands

CI runs these, and all must pass before a push:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p bms-core --target thumbv7em-none-eabihf   # once: rustup target add thumbv7em-none-eabihf
```

## Git

- Commit as the configured git user, with no `Co-Authored-By` or `Claude-Session` trailers in commit messages.
- When a decision changes, update `docs/PLAN.md` in the same commit.
