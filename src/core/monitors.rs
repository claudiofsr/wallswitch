use crate::{Dimension, Orientation};
use serde::{Deserialize, Serialize};

/// Monitor properties
///
/// Each monitor can have a diferent number of pictures (or images)
///
/// Each monitor can have different pictures (or images) orientation
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Monitor {
    /// Indicates how the images are combined
    pub picture_orientation: Orientation,
    /// Set number of pictures per monitor [default: 1]
    pub pictures_per_monitor: u8,
    /// Set the monitor resolution: "widthxheight"
    pub resolution: Dimension,
}

impl Default for Monitor {
    fn default() -> Self {
        Monitor {
            picture_orientation: Orientation::Horizontal,
            pictures_per_monitor: 1,
            resolution: Dimension::default(),
        }
    }
}

impl Monitor {
    pub fn flip(mut self) -> Self {
        match self.picture_orientation {
            Orientation::Horizontal => {
                self.picture_orientation = Orientation::Vertical;
            }
            Orientation::Vertical => {
                self.picture_orientation = Orientation::Horizontal;
            }
        };

        self
    }
}

/// Get N monitors
pub fn get_monitors(number: usize) -> Vec<Monitor> {
    let monitors: Vec<Monitor> = vec![Monitor::default(); number];

    monitors
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| {
            if index % 2 == 0 {
                monitor.flip()
            } else {
                monitor
            }
        })
        .collect()
}
