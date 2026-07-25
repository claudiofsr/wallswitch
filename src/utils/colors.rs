use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::ops::{Add, Mul, Sub};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BLACK: &str = "\x1b[30m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";

/// Terminal control character to return cursor to line start.
pub const CURSOR_TO_START: &str = "\r";
/// Terminal control character to erase line contents to the end.
pub const ERASE_LINE_TO_END: &str = "\x1b[K";

/// Trait extending string-like types with ANSI terminal color and formatting options.
pub trait Colors {
    fn bold(&self) -> String;
    fn dimmed(&self) -> String;
    fn black(&self) -> String;
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
    fn magenta(&self) -> String;
    fn cyan(&self) -> String;
    fn white(&self) -> String;
    fn to_line_start(&self) -> String;
}

impl<T> Colors for T
where
    T: Display,
{
    fn bold(&self) -> String {
        format!("{BOLD}{self}{RESET}")
    }
    fn dimmed(&self) -> String {
        format!("{DIM}{self}{RESET}")
    }
    fn black(&self) -> String {
        format!("{BLACK}{self}{RESET}")
    }
    fn red(&self) -> String {
        format!("{RED}{self}{RESET}")
    }
    fn green(&self) -> String {
        format!("{GREEN}{self}{RESET}")
    }
    fn yellow(&self) -> String {
        format!("{YELLOW}{self}{RESET}")
    }
    fn blue(&self) -> String {
        format!("{BLUE}{self}{RESET}")
    }
    fn magenta(&self) -> String {
        format!("{MAGENTA}{self}{RESET}")
    }
    fn cyan(&self) -> String {
        format!("{CYAN}{self}{RESET}")
    }
    fn white(&self) -> String {
        format!("{WHITE}{self}{RESET}")
    }
    fn to_line_start(&self) -> String {
        format!("{CURSOR_TO_START}{self}{ERASE_LINE_TO_END}")
    }
}

// ==============================================================================
// GRAPHICS AND IMAGE RENDERING COLOR MODELS
// ==============================================================================

/// Zero-cost, stack-allocated raw float-based RGB color vector.
///
/// Designed with double-precision floats to unify color calculations
/// with complex space viewport dimensions, eliminating type-casting overhead.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorRGB {
    /// Red channel value in the range [0.0, 1.0].
    pub red: f64,
    /// Green channel value in the range [0.0, 1.0].
    pub green: f64,
    /// Blue channel value in the range [0.0, 1.0].
    pub blue: f64,
}

impl ColorRGB {
    /// Creates a new ColorRGB vector.
    #[inline(always)]
    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
    }

    /// Creates a `ColorRGB` by reading 3 bytes from a slice, normalizing values to the range `[0.0, 1.0]`.
    ///
    /// # Panics
    ///
    /// This method will panic if the slice has fewer than 3 elements.
    #[inline(always)]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            red: slice[0] as f64 / 255.0,
            green: slice[1] as f64 / 255.0,
            blue: slice[2] as f64 / 255.0,
        }
    }

    /// Converts the internal normalized float channels back to the byte range `[0..255]`
    /// and writes them to a mutable 3-byte slice.
    ///
    /// # Panics
    ///
    /// This method will panic if the slice has fewer than 3 elements.
    #[inline(always)]
    pub fn write_to_slice(&self, slice: &mut [u8]) {
        slice[0] = (self.red * 255.0).clamp(0.0, 255.0) as u8;
        slice[1] = (self.green * 255.0).clamp(0.0, 255.0) as u8;
        slice[2] = (self.blue * 255.0).clamp(0.0, 255.0) as u8;
    }

    /// Performs linearly interpolated (LERP) blending with another ColorRGB.
    ///
    /// Interpolates such that returning `self` at t = 0.0 and `other` at t = 1.0.
    /// This formulation utilizes inlined algebraic operators for readability and performance.
    #[inline(always)]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        let t_clamp = t.clamp(0.0, 1.0);
        self + (other - self) * t_clamp
    }

    /// Shifts RGB channels cyclically to produce a secondary color.
    #[inline(always)]
    pub fn rotated(&self) -> Self {
        Self {
            red: self.green,
            green: self.blue,
            blue: self.red,
        }
    }

    /// Converts the struct's color channels into a raw float array [red, green, blue].
    #[inline(always)]
    pub const fn to_array(&self) -> [f64; 3] {
        [self.red, self.green, self.blue]
    }

    /// Returns the maximum component value among the Red, Green, and Blue channels.
    #[inline(always)]
    pub fn max_component(&self) -> f64 {
        self.red.max(self.green).max(self.blue)
    }

    /// Normalizes the color components by dividing each channel by the maximum channel value,
    /// boosting visual intensity to its peak.
    #[inline(always)]
    pub fn saturate_components(&self) -> Self {
        let max = self.max_component();
        if max > 0.0 {
            Self {
                red: self.red / max,
                green: self.green / max,
                blue: self.blue / max,
            }
        } else {
            *self
        }
    }

    /// Scales all color channels by a single floating-point factor.
    #[inline(always)]
    pub fn scale(self, factor: f64) -> Self {
        self * factor
    }

    /// Clamps all color channels to the standard range [0.0, 1.0].
    #[inline(always)]
    pub fn clamp_bounds(&self) -> Self {
        Self {
            red: self.red.clamp(0.0, 1.0),
            green: self.green.clamp(0.0, 1.0),
            blue: self.blue.clamp(0.0, 1.0),
        }
    }

    /// Computes the component-wise square of the color components.
    #[inline(always)]
    pub fn squared(&self) -> Self {
        Self {
            red: self.red * self.red,
            green: self.green * self.green,
            blue: self.blue * self.blue,
        }
    }

    /// Computes the component-wise square root of the color components.
    #[inline(always)]
    pub fn sqrt(&self) -> Self {
        Self {
            red: self.red.sqrt(),
            green: self.green.sqrt(),
            blue: self.blue.sqrt(),
        }
    }

    /// Calculates the perceived relative luminance using ITU-R BT.709 coefficients.
    #[inline(always)]
    pub fn luminance(self) -> f64 {
        self.red
            .mul_add(0.2126, self.green.mul_add(0.7152, self.blue * 0.0722))
    }

    /// Clamps components to the valid normalized color range boundary.
    #[inline(always)]
    pub fn saturate(self) -> Self {
        Self {
            red: self.red.clamp(0.0, 1.0),
            green: self.green.clamp(0.0, 1.0),
            blue: self.blue.clamp(0.0, 1.0),
        }
    }

    /// Linearizes the sRGB gamma curve using a fast power of 2 approximation.
    #[inline(always)]
    pub fn gamma2(self) -> Self {
        Self {
            red: self.red * self.red,
            green: self.green * self.green,
            blue: self.blue * self.blue,
        }
    }

    /// Applies gamma compression using a square root approximation.
    #[inline(always)]
    pub fn ungamma2(self) -> Self {
        Self {
            red: self.red.sqrt(),
            green: self.green.sqrt(),
            blue: self.blue.sqrt(),
        }
    }

    /// Gamma-corrected blend.
    #[inline(always)]
    pub fn blend(self, other: Self, alpha: f64) -> Self {
        self.gamma2().lerp(other.gamma2(), alpha).ungamma2()
    }

    /// Creates a ColorRGB with equal values across all three channels.
    #[inline(always)]
    pub fn splat(v: f64) -> Self {
        Self::new(v, v, v)
    }

    /// Returns the complementary (inverse) color of the current vector.
    /// This formulation utilizes standard algebraic scalar subtraction.
    #[inline(always)]
    pub fn complementary(self) -> Self {
        1.0 - self
    }

    /// Scales and saturates all color channels by a single floating-point factor.
    #[inline(always)]
    pub fn boost(self, factor: f64) -> Self {
        (self * factor).saturate()
    }
}

// ==============================================================================
// OPERATOR OVERLOADING TRAIT IMPLEMENTATIONS
// ==============================================================================

impl Add for ColorRGB {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self::Output {
        Self {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }
}

impl Sub for ColorRGB {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        Self {
            red: self.red - other.red,
            green: self.green - other.green,
            blue: self.blue - other.blue,
        }
    }
}

impl Sub<ColorRGB> for f64 {
    type Output = ColorRGB;

    #[inline(always)]
    fn sub(self, other: ColorRGB) -> Self::Output {
        ColorRGB {
            red: self - other.red,
            green: self - other.green,
            blue: self - other.blue,
        }
    }
}

impl Mul<f64> for ColorRGB {
    type Output = Self;

    #[inline(always)]
    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            red: self.red * scalar,
            green: self.green * scalar,
            blue: self.blue * scalar,
        }
    }
}

impl Mul for ColorRGB {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        )
    }
}

/// Represents a named neon color palette with float-based RGB channels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeonColor {
    /// The raw mathematical RGB channels.
    pub color_rgb: ColorRGB,
    /// Descriptive name of the color palette.
    pub name: &'static str,
}

impl NeonColor {
    /// Shifts RGB channels cyclically (R→G→B→R).
    #[inline(always)]
    pub fn rotated_color(&self) -> ColorRGB {
        self.color_rgb.rotated()
    }

    /// Linearly interpolates (LERP) blending with another NeonColor.
    #[inline(always)]
    pub fn lerp(&self, other: &Self, t: f64) -> ColorRGB {
        self.color_rgb.lerp(other.color_rgb, t)
    }

    /// Converts the struct's color channels into a raw float array [red, green, blue].
    #[inline(always)]
    pub const fn to_array(&self) -> [f64; 3] {
        self.color_rgb.to_array()
    }
}

impl fmt::Display for NeonColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RGB [{:.2}, {:.2}, {:.2}] {}",
            self.color_rgb.red, self.color_rgb.green, self.color_rgb.blue, self.name
        )
    }
}

pub const NEON_PALETTES: &[NeonColor] = &[
    NeonColor {
        color_rgb: ColorRGB {
            red: 1.0,
            green: 0.05,
            blue: 0.55,
        },
        name: "Neon Rose - Punchy Pink",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.0,
            green: 0.95,
            blue: 0.85,
        },
        name: "Vivid Turquoise - Electric Teal",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.95,
            green: 0.45,
            blue: 0.0,
        },
        name: "Tangerine Dream - Electric Orange",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.4,
            green: 0.9,
            blue: 0.0,
        },
        name: "Radioactive Lime - Acid Green",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.9,
            green: 0.9,
            blue: 0.0,
        },
        name: "Laser Lemon - Bright Yellow",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.05,
            green: 0.8,
            blue: 1.0,
        },
        name: "Blue Bolt - Intense Sky Cyan",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.85,
            green: 0.15,
            blue: 1.0,
        },
        name: "Radiant Orchid - Electric Violet",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 1.0,
            green: 0.35,
            blue: 0.1,
        },
        name: "Fiery Coral - Sunset Flame",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.1,
            green: 0.95,
            blue: 0.4,
        },
        name: "Mint Spark - Spring Neon Green",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 1.0,
            green: 0.0,
            blue: 0.35,
        },
        name: "Cherry Flare - Electric Ruby",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.75,
            green: 0.95,
            blue: 0.0,
        },
        name: "Vibrant Chartreuse - Pear Glow",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.0,
            green: 1.0,
            blue: 0.7,
        },
        name: "Aqua Glow - Supercharged seafoam",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 1.0,
            green: 0.55,
            blue: 0.4,
        },
        name: "Warm Apricot - Peach Spark",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.9,
            green: 0.2,
            blue: 0.9,
        },
        name: "Cyberpunk Fuchsia - Hot Magenta",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.35,
            green: 1.0,
            blue: 0.5,
        },
        name: "Bright Emerald - Neon Clover",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 1.0,
            green: 0.85,
            blue: 0.0,
        },
        name: "Solar Flare - Amber Gold",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.55,
            green: 0.35,
            blue: 1.0,
        },
        name: "Electric Amethyst - Bright Violet",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 1.0,
            green: 0.25,
            blue: 0.5,
        },
        name: "Flamingo Neon - Warm Pink",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.2,
            green: 0.9,
            blue: 0.9,
        },
        name: "Glacier Cyan - Ice Blue Glow",
    },
    NeonColor {
        color_rgb: ColorRGB {
            red: 0.85,
            green: 1.0,
            blue: 0.2,
        },
        name: "Radioactive Melon - Lemon Lime Flare",
    },
];
