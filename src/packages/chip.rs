use crate::config::Config;
use crate::drawing::{Box3D, Drawing};
use crate::error::*;

use super::{calc::Ipc7351B, PackageHandler, TwoPin};

pub struct ChipPackage {}

impl ChipPackage {
    pub fn new() -> Self {
        ChipPackage {}
    }
}

impl PackageHandler for ChipPackage {
    fn draw_pattern(&self, config: &Config, lib_config: &Config) -> Result<Drawing> {
        debug!("draw chip pattern");

        let body_size_x = config.get_range("package.body-size-x")?;
        let body_size_y = config.get_range("package.body-size-y")?;
        let body_size_z = if let Some(z) = config.get_range("package.body-size-z").ok() {
            z
        } else {
            if let Some(z) = config.get_range("package.size-z").ok() {
                z
            } else {
                bail!(QedaError::MissingDimension(
                    "'package' should have either 'body_size_z' or 'size_z'"
                ));
            }
        };
        let lead_len = config.get_range("package.lead-length")?;

        let ipc = Ipc7351B::default()
            .lead_span(body_size_x)
            .lead_width(body_size_y)
            .lead_height(body_size_z)
            .lead_len(lead_len)
            .goals("chip", "N") // TODO: Replace density level by the config value
            .tols(0.05, 0.025) // TODO: Replace fabrication and placement tolerances by the config values
            .calc();

        let mut two_pin = TwoPin::default();
        two_pin.pad_size = ipc.pad_size;
        two_pin.pad_distance = ipc.pad_distance;
        two_pin.courtyard = ipc.courtyard;

        let mut drawing = Drawing::new();
        two_pin.draw(&mut drawing, lib_config)?;
        Ok(drawing)
    }

    fn draw_model(&self, _config: &Config, _lib_config: &Config) -> Result<Drawing> {
        debug!("draw chip model");
        let mut drawing = Drawing::new();
        drawing.add_box3d(Box3D::new().origin(0.0, 1.0, 2.0).dimensions(3.0, 4.0, 5.0));
        Ok(drawing)
    }
}