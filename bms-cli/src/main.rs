//! Headless scenario runs: baseline against improved, written to CSV.

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};

use bms_core::cell::{CellParams, CellState};

fn main() -> io::Result<()> {
    discharge_1c()
}

/// Discharges a full HG2-like cell at 1C (3 A) to 3.0 V in 1 s steps and
/// writes t, i, v and SoC to `out/discharge_1c.csv`.
fn discharge_1c() -> io::Result<()> {
    let params = CellParams::HG2;
    let current = 3.0; // A
    let dt = 1.0; // s
    let cutoff = 3.0; // V
    let path = "out/discharge_1c.csv";

    fs::create_dir_all("out")?;
    let mut csv = BufWriter::new(File::create(path)?);
    writeln!(csv, "t_s,i_a,v_v,soc")?;

    let mut cell = CellState::rested(1.0);
    let mut t = 0.0;
    loop {
        let v = cell.terminal_voltage(&params, current);
        writeln!(csv, "{t},{current},{v:.6},{:.6}", cell.soc)?;
        if v <= cutoff {
            break;
        }
        cell.step(&params, current, dt);
        t += dt;
    }
    csv.flush()?;

    println!(
        "1C discharge to {cutoff:.1} V: {t} s ({:.1} min), ending at {:.2} % SoC; wrote {path}",
        t / 60.0,
        cell.soc * 100.0
    );
    Ok(())
}
