pub use std::str::FromStr;

use crate::error::Result;

#[derive(Clone, Debug)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

impl Default for HAlign {
    #[inline]
    fn default() -> Self {
        Self::Left
    }
}

impl FromStr for HAlign {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "left" => HAlign::Left,
            "right" => HAlign::Right,
            "center" => HAlign::Center,
            _ => HAlign::default(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

impl Default for VAlign {
    #[inline]
    fn default() -> Self {
        Self::Bottom
    }
}

impl FromStr for VAlign {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "bottom" => VAlign::Bottom,
            "top" => VAlign::Top,
            "middle" => VAlign::Middle,
            _ => VAlign::default(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    #[inline]
    fn default() -> Self {
        Orientation::Horizontal
    }
}

#[derive(Clone, Debug)]
pub struct Visibility(pub bool);

impl Default for Visibility {
    #[inline]
    fn default() -> Self {
        Visibility(true)
    }
}

#[derive(Debug)]
pub enum PinDirection {
    Up,
    Down,
    Right,
    Left,
}

bitflags! {
    #[derive(Default)]
    pub struct Layer: u32 {
        const NONE              = 0x00000000;
        const COPPER_TOP        = 0x00000001;
        const COPPER_BOTTOM     = 0x00000002;
        const SILKSCREEN_TOP    = 0x00000004;
        const SILKSCREEN_BOTTOM = 0x00000008;
        const MASK_TOP          = 0x00000010;
        const MASK_BOTTOM       = 0x00000020;
        const PASTE_TOP         = 0x00000040;
        const PASTE_BOTTOM      = 0x00000080;
        const ASSEMBLY_TOP      = 0x00000100;
        const ASSEMBLY_BOTTOM   = 0x00000200;
        const COURTYARD_TOP     = 0x00000400;
        const COURTYARD_BOTTOM  = 0x00000800;
        const BOARD             = 0x10000000;
    }
}
