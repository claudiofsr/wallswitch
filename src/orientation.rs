use crate::WSError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Orientation: Horizontal or Vertical.
///
/// Default: Horizontal.
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Orientation {
    type Err = WSError<'static>;

    fn from_str(s: &str) -> Result<Self, WSError<'static>> {
        match s.trim().to_lowercase().as_str() {
            "horizontal" => Ok(Self::Horizontal),
            "vertical" => Ok(Self::Vertical),
            _ => Err(WSError::InvalidOrientation),
        }
    }
}
