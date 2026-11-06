# rustbms

A battery management system written twice: a baseline built from conventional algorithms, and an improved version that has to beat it on the same simulated scenarios. The simulated pack is modeled on published figures for the first-generation Porsche Taycan Performance Battery Plus: 198s2p in 33 modules, 723 V nominal, 610 to 835 V.

This project is not affiliated with Porsche. Porsche's BMS software is not public, so every result compares the improved BMS with this repo's own baseline.

## Layout

| Path | What it is |
| --- | --- |
| `bms-core` | ECU logic, `no_std`: cell model, SoC estimation, protection, contactors, balancing |
| `bms-cli` | headless scenario runs to CSV, later the baseline against improved report |
| `dbc/bms.dbc` | the CAN interface |
| `docs/PLAN.md` | the plan, the milestones and the numbers the tests check |

`bms-sim` (pack plant and sensors), `bms-can` (DBC code generation and buses) and `bms-app` (egui dashboard) arrive with their milestones.

## Build

```sh
cargo test --workspace
rustup target add thumbv7em-none-eabihf
cargo build -p bms-core --target thumbv7em-none-eabihf   # proves bms-core needs no std
```
