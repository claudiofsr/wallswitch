//! Procedural Starfield / Bokeh overlay generator.
//!
//! This module projects soft, circular glowing stars and light orbs of varying sizes,
//! intensities, and high-contrast neon colors over active background canvases.
//! Rendering and blending equations are processed in parallel using a gamma-corrected
//! linear color space model to prevent gray-fringe artifacts.

use crate::{
    ColorRGB, Complex, ImageEffect, NEON_PALETTES, NeonColor, RandomExt, WallSwitchResult,
    get_random_integer, process_rows_parallel_scoped,
};
use image::RgbImage;

/// The minimum and maximum limits for the randomized star count.
pub const STAR_RANGE: [usize; 2] = [60, 120];

/// Represents a single procedurally generated star element.
pub struct Star {
    /// The coordinate position of the star center in complex space.
    pub position: Complex,
    /// The physical radius of the star.
    pub radius: f64,
    /// The visual peak intensity.
    pub intensity: f64,
    /// The neon color palette assigned to the star.
    pub color_palette: NeonColor,
}

/// A procedural generator for rendering cosmic starfields and bokeh onto image backgrounds.
pub struct StarfieldGenerator {
    /// The collection of active star elements.
    pub stars: Vec<Star>,
}

impl ImageEffect for StarfieldGenerator {
    /// Blends the generated starfield overlay onto the image buffer in parallel.
    fn apply(&self, rgb_img: &mut RgbImage) {
        let contrast_color = ColorRGB::new(0.64, 0.75, 0.85);

        process_rows_parallel_scoped(rgb_img, |y, row_data| {
            let y_f = y as f64;

            let mut active_stars = Vec::with_capacity(16);
            for star in &self.stars {
                let dy = star.position.im - y_f;
                let limit = star.radius * 2.0;
                if dy.abs() < limit {
                    let dy_sq = dy * dy;
                    let star_radius_sq = star.radius * star.radius;
                    active_stars.push((star, dy_sq, star_radius_sq));
                }
            }

            for (x, pixel_slice) in row_data.chunks_exact_mut(3).enumerate() {
                let x_f = x as f64;

                let mut color_acc = ColorRGB::default();
                let mut total_alpha = 0.0;

                for &(star, dy_sq, star_radius_sq) in &active_stars {
                    let dx = star.position.re - x_f;
                    let dist_sq = dx * dx + dy_sq;

                    if dist_sq < star_radius_sq * 4.0 {
                        let factor = (-dist_sq / (2.0 * star_radius_sq)).exp();
                        let alpha = factor * star.intensity;

                        // Interpolates and accumulates directly using ColorRGB operations
                        let star_color = star.color_palette.color_rgb.lerp(contrast_color, 0.25);
                        color_acc = color_acc + star_color * alpha;
                        total_alpha += alpha;
                    }
                }

                if total_alpha > 0.001 {
                    let alpha_clamp = total_alpha.min(0.95);
                    let fg_color = color_acc * (1.0 / total_alpha);

                    let bg = ColorRGB::from_slice(pixel_slice);
                    let blended = bg.blend(fg_color, alpha_clamp);
                    blended.write_to_slice(pixel_slice);
                }
            }
        });
    }

    /// Returns a formatting diagnostic string about the active generator.
    fn info(&self) -> String {
        format!("overlay ({} stars)", self.stars.len())
    }
}

impl StarfieldGenerator {
    /// Generates a randomized field of stars based on the monitor's physical dimensions.
    ///
    /// Reuses [`new`](Self::new) internally to construct the star positions and color properties.
    ///
    /// # Errors
    ///
    /// Returns [`WallSwitchError::EmptySlice`](crate::WallSwitchError::EmptySlice)
    /// if the global neon palette table is empty.
    pub fn random(monitor: &crate::Monitor) -> WallSwitchResult<Self> {
        let width = monitor.resolution.width;
        let height = monitor.resolution.height;

        let count = get_random_integer(STAR_RANGE[0], STAR_RANGE[1]);
        Self::new(count, width, height)
    }

    /// Generates a randomized field of stars using the centralized high-contrast neon colors.
    ///
    /// # Errors
    ///
    /// Returns [`WallSwitchError::EmptySlice`](crate::WallSwitchError::EmptySlice)
    /// if the global neon palette table is empty.
    pub fn new(count: usize, width: u64, height: u64) -> WallSwitchResult<Self> {
        let mut stars = Vec::with_capacity(count);

        for _ in 0..count {
            let x: f64 = get_random_integer(0, width);
            let y: f64 = get_random_integer(0, height);
            let radius: f64 = get_random_integer(5, 45);
            let intensity = get_random_integer::<_, f64>(30, 95) / 100.0;

            let color_palette = NEON_PALETTES.get_random_sample()?;

            stars.push(Star {
                position: Complex::new(x, y),
                radius,
                intensity,
                color_palette,
            });
        }

        Ok(Self { stars })
    }
}

#[cfg(test)]
mod tests_star {
    use super::*;
    use crate::core::Monitor;

    #[test]
    fn test_star_generation() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let starfield = StarfieldGenerator::random(&monitor)?;
        assert!(!starfield.stars.is_empty());
        Ok(())
    }
}
