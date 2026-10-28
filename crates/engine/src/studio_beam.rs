//! Analytic rectangular cantilever reference, independent of CAD and external solvers.
//! Units: mm, N, MPa (N/mm²), kg/m³. Linear Euler–Bernoulli bending only.

#[derive(Clone, Copy, Debug)]
pub struct Beam {
    pub length_mm: f64,
    pub thickness_mm: f64,
    pub height_mm: f64,
    pub young_mpa: f64,
    pub density_kg_m3: f64,
    pub force_n: f64,
    pub deflection_limit_mm: f64,
}

impl Default for Beam {
    fn default() -> Self {
        Self {
            length_mm: 1000.,
            thickness_mm: 10.,
            height_mm: 100.,
            young_mpa: 210000.,
            density_kg_m3: 7850.,
            force_n: 1000.,
            deflection_limit_mm: 2.,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BeamResult {
    pub mass_kg: f64,
    pub second_moment_mm4: f64,
    pub tip_deflection_mm: f64,
    pub root_stress_mpa: f64,
    pub stiffness_n_m: f64,
    pub strain_energy_j: f64,
    pub slenderness: f64,
    pub deflection_ratio: f64,
    pub feasible: bool,
}

impl Beam {
    pub fn evaluate(self) -> std::result::Result<BeamResult, &'static str> {
        for (value, min, max) in [
            (self.length_mm, 0.001, 1e6),
            (self.thickness_mm, 0.001, 1e5),
            (self.height_mm, 0.001, 1e5),
            (self.young_mpa, 0.001, 1e7),
            (self.density_kg_m3, 0.001, 1e6),
            (self.force_n, -1e9, 1e9),
            (self.deflection_limit_mm, 0.000001, 1e6),
        ] {
            if !value.is_finite() || value < min || value > max {
                return Err("beam inputs must be finite and within the documented engineering bounds");
            }
        }
        let second_moment_mm4 = self.thickness_mm * self.height_mm.powi(3) / 12.;
        let compliance_mm_n = self.length_mm.powi(3) / (3. * self.young_mpa * second_moment_mm4);
        let tip_deflection_mm = self.force_n * compliance_mm_n;
        let result = BeamResult {
            mass_kg: self.density_kg_m3 * self.length_mm * self.thickness_mm * self.height_mm * 1e-9,
            second_moment_mm4,
            tip_deflection_mm,
            root_stress_mpa: self.force_n.abs() * self.length_mm * self.height_mm / (2. * second_moment_mm4),
            stiffness_n_m: 1000. / compliance_mm_n,
            strain_energy_j: 0.5 * self.force_n * tip_deflection_mm * 0.001,
            slenderness: self.length_mm / self.height_mm,
            deflection_ratio: tip_deflection_mm.abs() / self.length_mm,
            feasible: tip_deflection_mm.abs() <= self.deflection_limit_mm,
        };
        if ![result.mass_kg, result.second_moment_mm4, result.tip_deflection_mm, result.root_stress_mpa, result.stiffness_n_m, result.strain_energy_j]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err("non-finite beam result");
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangular_beam_matches_hand_calculation_and_scaling() {
        let beam = Beam::default();
        let r = beam.evaluate().unwrap();
        assert!((r.mass_kg - 7.85).abs() < 1e-12);
        assert!((r.tip_deflection_mm - 40. / 21.).abs() < 1e-12);
        assert!((r.root_stress_mpa - 60.).abs() < 1e-12);
        let higher = Beam { height_mm: 200., ..beam }.evaluate().unwrap();
        assert!((higher.tip_deflection_mm * 8. - r.tip_deflection_mm).abs() < 1e-12);
        let reversed = Beam { force_n: -1000., ..beam }.evaluate().unwrap();
        assert_eq!(r.tip_deflection_mm, -reversed.tip_deflection_mm);
        assert_eq!(r.strain_energy_j, reversed.strain_energy_j);
        assert!(Beam { height_mm: 0., ..beam }.evaluate().is_err());
        assert!(Beam { young_mpa: f64::NAN, ..beam }.evaluate().is_err());
    }
}
