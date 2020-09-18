mod chip;

use std::collections::HashMap;
use std::fmt::{self, Debug};

use crate::config::Config;
use crate::drawing::Drawing;
use crate::error::*;

use chip::ChipPackage;

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

pub trait PackageHandler {
    fn draw_pattern(&self, comp_cfg: &Config, lib_cfg: &Config) -> Result<Drawing>;
    fn draw_model(&self, comp_cfg: &Config, lib_cfg: &Config) -> Result<Drawing>;
}

impl Debug for dyn PackageHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PackageHandler")
    }
}

#[derive(Debug)]
pub struct Packages<'a> {
    handlers: HashMap<&'a str, Box<dyn PackageHandler>>,
}

impl<'a> Packages<'a> {
    pub fn new() -> Self {
        let mut handlers: HashMap<&'a str, Box<dyn PackageHandler>> = HashMap::new();
        handlers.insert("chip", Box::new(ChipPackage::new()));

        Packages { handlers }
    }

    pub fn get_handler(&self, key: &str) -> Result<&Box<dyn PackageHandler>> {
        self.handlers
            .get(key)
            .ok_or(QedaError::InvalidPackageType(key.to_string()).into())
    }
}
