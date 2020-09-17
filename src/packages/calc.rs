use crate::config::{Config, Range};

#[derive(Debug)]
pub enum PackageType {
    Unknown,
    Chip,
}

impl Default for PackageType {
    fn default() -> Self {
        PackageType::Unknown
    }
}

#[derive(Debug, Default)]
pub struct Ipc7351B {
    pub pad_distance: f64,
    pub pad_size: (f64, f64),
    pub courtyard: f64,

    package_type: PackageType,
    lead_span: Range,
    lead_len: Range,
    lead_width: Range,
    lead_height: Range,
    body: Option<f64>,
    pitch: Option<f64>,

    toe: f64,
    heel: f64,
    side: f64,
    fab_tol: f64,
    place_tol: f64,

    clearance: f64,
    space_for_iron: f64,

    always_calc: bool,
}

impl Ipc7351B {
    /// Creates an empty Ipc7351B.
    pub fn new(package_type: PackageType) -> Self {
        Ipc7351B {
            package_type: package_type,
            ..Self::default()
        }
    }

    /// Sets a lead span, i.e. a distance from the one lead edge to the opposite lead edge.
    pub fn lead_span(mut self, span: Range) -> Self {
        self.lead_span = span;
        self
    }

    /// Sets the lead length.
    pub fn lead_len(mut self, len: Range) -> Self {
        self.lead_len = len;
        self
    }

    /// Sets the lead width.
    pub fn lead_width(mut self, width: Range) -> Self {
        self.lead_width = width;
        self
    }

    /// Sets the lead height.
    pub fn lead_height(mut self, height: Range) -> Self {
        self.lead_height = height;
        self
    }

    /// Calculates pad parameters.
    pub fn calc(mut self) -> Self {
        let span_tol = self.lead_span.tol();
        let len_tol = self.lead_len.tol();
        let width_tol = self.lead_width.tol();

        let s_min = self.lead_span.min() - 2.0 * self.lead_len.max();
        let s_max = self.lead_span.max() - 2.0 * self.lead_len.min();
        let s_tol = s_max - s_min;
        let s_tol_rms = (span_tol * span_tol + 2.0 * len_tol * len_tol).sqrt();
        let s_diff = s_tol - s_tol_rms;

        let new_s_min = s_min + s_diff / 2.0;
        let new_s_max = s_max - s_diff / 2.0;
        let new_s_tol = new_s_max - new_s_min;

        let toe_tol = (span_tol * span_tol
            + 4.0 * self.fab_tol * self.fab_tol
            + 4.0 * self.place_tol * self.place_tol)
            .sqrt();
        let heel_tol = (new_s_tol * new_s_tol
            + 4.0 * self.fab_tol * self.fab_tol
            + 4.0 * self.place_tol * self.place_tol)
            .sqrt();
        let side_tol = (width_tol * width_tol
            + 4.0 * self.fab_tol * self.fab_tol
            + 4.0 * self.place_tol * self.place_tol)
            .sqrt();

        let z_max = self.lead_span.min() + 2.0 * self.toe + toe_tol;
        let g_min = new_s_max - 2.0 * self.heel - heel_tol;
        let y_ref = self.lead_width.min() + 2.0 * self.side + side_tol;

        let mut pad_width = Self::round_size((z_max - g_min) / 2.0);
        let mut pad_height = Self::round_size(y_ref);
        let mut pad_distance = Self::round_place((z_max + g_min) / 2.0);

        let mut gap = pad_distance - pad_width;
        let span = pad_distance + pad_width;

        let mut trim = false;

        // Trim pads if they are too near one to another
        if gap < self.clearance {
            gap = self.clearance;
            trim = true;
        }

        // Trim if pad is under body
        if let Some(body) = self.body {
            if gap < (body - 0.1) {
                // TODO: determine, why 0.1
                gap = body - 0.1;
                trim = true;
            }
        }

        if trim {
            pad_width = Self::round_size((span - gap) / 2.0);
            pad_distance = Self::round_place((span + gap) / 2.0);
        }

        // Pad height should not violate clearance rules
        if let Some(pitch) = self.pitch {
            if pad_height > (pitch - self.clearance) {
                pad_height = pitch - self.clearance;
            }
        }

        self.pad_size = (pad_width, pad_height);
        self.pad_distance = pad_distance;

        self
    }

    /// Applies post processing according to the pattern config.
    pub fn post_proc(mut self, config: &Config) -> Self {
        if !self.always_calc {
            if let Some(pad_width) = config.get_f64("pattern.pad-size-x").ok() {
                self.pad_size.0 = pad_width;
            }
            if let Some(pad_height) = config.get_f64("pattern.pad-size-y").ok() {
                self.pad_size.1 = pad_height;
            }
            if let Some(pad_size) = config.get_pair("pattern.pad-size").ok() {
                self.pad_size = pad_size;
            }
            if let Some(pad_distance) = config.get_f64("pattern.pad-distance").ok() {
                self.pad_distance = pad_distance;
            }
            if let Some(pad_span) = config.get_f64("pattern.pad-span").ok() {
                self.pad_distance = pad_span - self.pad_size.0;
            }
            if let Some(pad_space) = config.get_f64("pattern.pad-space").ok() {
                self.pad_distance = pad_space + self.pad_size.0;
            }
        }
        if self.space_for_iron > 0.0 {
            let lead_to_pad = (self.pad_distance + self.pad_size.0 - self.lead_span.nom()) / 2.0;
            if lead_to_pad < self.space_for_iron {
                let d = self.space_for_iron - lead_to_pad;
                self.pad_size.0 += d;
                self.pad_distance += d;
            }
        }
        self
    }

    /// Gets settings from a config and applies them.
    pub fn settings(mut self, config: &Config) -> Self {
        self.fab_tol = config
            .get_f64("pattern.tolerance.fabrication")
            .unwrap_or(0.05);
        self.place_tol = config
            .get_f64("pattern.tolerance.placement")
            .unwrap_or(0.025);
        self.clearance = config
            .get_f64("pattern.clearance.pad-to-pad")
            .unwrap_or(0.0);
        self.space_for_iron = config
            .get_f64("pattern.minimum.space-for-iron")
            .unwrap_or(0.0);
        self.always_calc = config.get_bool("always-calculate").unwrap_or(false);

        self.density_level(config.get_str("pattern.density-level").unwrap_or("N"))
    }
}

// Private methods
impl Ipc7351B {
    fn density_level(mut self, density_level: &str) -> Self {
        let i = match density_level {
            "M" | "m" | "most" => 0,
            "L" | "l" | "least" => 2,
            _ => 1, // "N", "nominal"
        };
        let (toe, heel, side, courtyard) = match self.package_type {
            PackageType::Chip => {
                let len = self.lead_span.nom(); //(self.lead_span.0 + self.lead_span.1) / 2.0;
                if len <= 0.5 {
                    // 01005 & Less
                    (
                        vec![0.06, 0.05, 0.04][i],    // Toe
                        vec![-0.02, -0.03, -0.04][i], // Heel
                        vec![-0.02, -0.03, -0.04][i], // Side
                        vec![0.2, 0.15, 0.1][i],      // Coutyard
                    )
                } else if len <= 0.75 {
                    // 0201
                    (
                        vec![0.12, 0.1, 0.08][i],     // Toe
                        vec![-0.01, -0.02, -0.03][i], // Heel
                        vec![-0.01, -0.02, -0.03][i], // Side
                        vec![0.2, 0.15, 0.1][i],      // Coutyard
                    )
                } else if len <= 1.3 {
                    // 0402, 0306 & 0502
                    (
                        vec![0.25, 0.2, 0.15][i],   // Toe
                        vec![0.0, -0.01, -0.02][i], // Heel
                        vec![0.0, -0.01, -0.02][i], // Side
                        vec![0.2, 0.15, 0.1][i],    // Coutyard
                    )
                } else if len <= 2.85 {
                    // 0603, 0705 & 0805
                    (
                        vec![0.4, 0.3, 0.2][i],    // Toe
                        vec![0.0, 0.0, 0.0][i],    // Heel
                        vec![0.05, 0.0, -0.05][i], // Side
                        vec![0.4, 0.2, 0.1][i],    // Coutyard
                    )
                } else if len <= 3.85 {
                    // 1206, 1210 & 0612
                    (
                        vec![0.45, 0.35, 0.25][i], // Toe
                        vec![0.0, 0.0, 0.0][i],    // Heel
                        vec![0.05, 0.0, -0.05][i], // Side
                        vec![0.4, 0.2, 0.1][i],    // Coutyard
                    )
                } else if len <= 4.75 {
                    // 1812 & 1825
                    (
                        vec![0.5, 0.4, 0.3][i],    // Toe
                        vec![0.0, 0.0, 0.0][i],    // Heel
                        vec![0.05, 0.0, -0.05][i], // Side
                        vec![0.4, 0.2, 0.1][i],    // Coutyard
                    )
                } else {
                    // 2010 & Greater
                    (
                        vec![0.6, 0.5, 0.4][i],    // Toe
                        vec![0.0, 0.0, 0.0][i],    // Heel
                        vec![0.05, 0.0, -0.05][i], // Side
                        vec![0.4, 0.2, 0.1][i],    // Coutyard
                    )
                }
            }
            _ => (
                vec![0.55, 0.35, 0.15][i], // Toe
                vec![0.45, 0.35, 0.25][i], // Heel
                vec![0.05, 0.03, 0.01][i], // Side
                vec![0.5, 0.25, 0.12][i],  // Coutyard
            ),
        };
        self.toe = toe;
        self.heel = heel;
        self.side = side;
        self.courtyard = courtyard;
        self
    }

    fn round_place(value: f64) -> f64 {
        let factor = 0.02;
        (value / factor).round() * factor
    }

    fn round_size(value: f64) -> f64 {
        let factor = 0.01;
        (value / factor).round() * factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc() {
        // Use calculator from pcblibraries.com for validation
        let ipc = Ipc7351B::new(PackageType::Unknown)
            .lead_span(Range(5.85, 6.2))
            .lead_width(Range(0.31, 0.51))
            .lead_len(Range(0.4, 1.27))
            .settings(&Config::new())
            .calc();

        assert_eq!(ipc.pad_distance, 4.96);
        assert_eq!(ipc.pad_size, (1.95, 0.6));
    }
}
