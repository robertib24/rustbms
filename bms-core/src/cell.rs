//! Single-cell model: a 2RC equivalent circuit on a tabulated OCV curve.
//!
//! The equations are in `docs/PLAN.md`, under "Reference cell and pack model".
//! The parameters are an argument of every call rather than part of the state,
//! so the simulated plant can run each cell with its own parameters while the
//! EKF runs the same equations with nominal ones.

/// Open-circuit voltage against state of charge, as `(soc, ocv_v)` points.
///
/// An illustrative NMC shape, not measured data; M6 replaces it with a fit to
/// the HG2 dataset. Keep the last point at the 4.20 V charge voltage: a table
/// that ends below it lets a CC-CV charge run far past 100 %.
pub const OCV_TABLE: [(f64, f64); 13] = [
    (0.00, 3.00),
    (0.05, 3.40),
    (0.10, 3.52),
    (0.20, 3.60),
    (0.30, 3.65),
    (0.40, 3.70),
    (0.50, 3.75),
    (0.60, 3.82),
    (0.70, 3.90),
    (0.80, 3.98),
    (0.90, 4.07),
    (0.95, 4.13),
    (1.00, 4.20),
];

/// Open-circuit voltage at `soc`, V, interpolated linearly in [`OCV_TABLE`].
///
/// Outside the table the end segments are extended rather than clamped, so the
/// voltage keeps rising past full and falling past empty.
pub fn ocv(soc: f64) -> f64 {
    // Segment k joins points k and k + 1. The number of inner points at or
    // below `soc` is k, and it stays on an end segment when `soc` is off the
    // table.
    let k = OCV_TABLE[1..OCV_TABLE.len() - 1]
        .iter()
        .filter(|(s, _)| *s <= soc)
        .count();
    let (s0, v0) = OCV_TABLE[k];
    let (s1, v1) = OCV_TABLE[k + 1];
    v0 + (v1 - v0) * (soc - s0) / (s1 - s0)
}

/// Parameters of the 2RC equivalent circuit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellParams {
    /// Capacity, Ah.
    pub capacity_ah: f64,
    /// Series resistance, Ω.
    pub r0: f64,
    /// Resistance of the fast RC pair, Ω.
    pub r1: f64,
    /// Time constant of the fast RC pair, s.
    pub tau1: f64,
    /// Resistance of the slow RC pair, Ω.
    pub r2: f64,
    /// Time constant of the slow RC pair, s.
    pub tau2: f64,
}

impl CellParams {
    /// The 3 Ah reference cell, sized like the LG INR18650HG2. The capacity is
    /// the datasheet value; the resistances and time constants are illustrative.
    pub const HG2: Self = Self {
        capacity_ah: 3.0,
        r0: 0.020,
        r1: 0.015,
        tau1: 15.0,
        r2: 0.015,
        tau2: 300.0,
    };
}

/// What changes as a cell runs: its SoC and the voltage across each RC pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellState {
    /// State of charge as a fraction: 1 is full, 0 is empty.
    pub soc: f64,
    /// Voltage across the fast RC pair, V.
    pub v1: f64,
    /// Voltage across the slow RC pair, V.
    pub v2: f64,
}

impl CellState {
    /// A cell at `soc` after a long rest, with both RC pairs relaxed.
    pub const fn rested(soc: f64) -> Self {
        Self {
            soc,
            v1: 0.0,
            v2: 0.0,
        }
    }

    /// Advances the state by `dt` seconds at current `i`, A (`i > 0` discharges).
    ///
    /// The step is exact for a current held constant over `dt`.
    pub fn step(&mut self, params: &CellParams, i: f64, dt: f64) {
        let a1 = libm::exp(-dt / params.tau1);
        let a2 = libm::exp(-dt / params.tau2);
        self.soc -= dt * i / (3600.0 * params.capacity_ah);
        self.v1 = a1 * self.v1 + params.r1 * (1.0 - a1) * i;
        self.v2 = a2 * self.v2 + params.r2 * (1.0 - a2) * i;
    }

    /// Terminal voltage while current `i` flows, V.
    pub fn terminal_voltage(&self, params: &CellParams, i: f64) -> f64 {
        ocv(self.soc) - params.r0 * i - self.v1 - self.v2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // The expected values are the checks table in docs/PLAN.md, which rounds
    // them; each tolerance is half a unit in the last digit the table prints.

    const HG2: CellParams = CellParams::HG2;

    /// Holds current `i` on `cell` for `seconds` steps of 1 s.
    fn hold(cell: &mut CellState, i: f64, seconds: u32) {
        for _ in 0..seconds {
            cell.step(&HG2, i, 1.0);
        }
    }

    /// Discharges a full, rested cell at 1C (3 A) in 1 s steps until its
    /// voltage under load reaches 3.0 V; returns the cell and the seconds taken.
    fn discharge_1c_to_cutoff() -> (CellState, u32) {
        let mut cell = CellState::rested(1.0);
        let mut seconds = 0;
        while cell.terminal_voltage(&HG2, 3.0) > 3.0 {
            cell.step(&HG2, 3.0, 1.0);
            seconds += 1;
        }
        (cell, seconds)
    }

    #[test]
    fn ocv_interpolates_and_extends_the_end_segments() {
        assert_relative_eq!(ocv(0.55), 3.785, epsilon = 1e-12);
        assert_relative_eq!(ocv(1.05), 4.27, epsilon = 1e-12);
        assert_relative_eq!(ocv(-0.05), 2.60, epsilon = 1e-12);
    }

    #[test]
    fn rest_voltage_at_100_percent() {
        let cell = CellState::rested(1.0);
        assert_relative_eq!(cell.terminal_voltage(&HG2, 0.0), 4.200, epsilon = 5e-4);
    }

    #[test]
    fn instant_drop_at_3a_is_r0_times_i() {
        let cell = CellState::rested(1.0);
        let drop = cell.terminal_voltage(&HG2, 0.0) - cell.terminal_voltage(&HG2, 3.0);
        assert_relative_eq!(drop, 0.060, epsilon = 5e-4);
    }

    #[test]
    fn discharge_at_1c_reaches_3v_after_3533_s() {
        let (cell, seconds) = discharge_1c_to_cutoff();
        assert_eq!(seconds, 3533);
        assert_relative_eq!(cell.soc, 0.0186, epsilon = 5e-5);
    }

    #[test]
    fn relaxation_after_the_cutoff() {
        let (mut cell, _) = discharge_1c_to_cutoff();
        assert_relative_eq!(cell.terminal_voltage(&HG2, 0.0), 3.059, epsilon = 5e-4);
        hold(&mut cell, 0.0, 15);
        assert_relative_eq!(cell.terminal_voltage(&HG2, 0.0), 3.090, epsilon = 5e-4);
        hold(&mut cell, 0.0, 3600 - 15); // 1 h after the cutoff
        assert_relative_eq!(cell.terminal_voltage(&HG2, 0.0), 3.149, epsilon = 5e-4);
    }

    #[test]
    fn pulse_of_6a_for_10_s_from_50_percent() {
        let mut cell = CellState::rested(0.5);
        assert_relative_eq!(cell.terminal_voltage(&HG2, 0.0), 3.7500, epsilon = 5e-5);
        assert_relative_eq!(cell.terminal_voltage(&HG2, 6.0), 3.6300, epsilon = 5e-5);
        hold(&mut cell, 6.0, 10);
        assert_relative_eq!(cell.terminal_voltage(&HG2, 6.0), 3.5805, epsilon = 5e-5);
    }
}
