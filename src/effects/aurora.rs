//! Cosmic Aurora wave generator overlay.
//!
//! Implements a procedural overlay mimicking polar aurora wave filaments using
//! multi-frequency wave mathematics. Per-row state is pre-computed outside the
//! inner pixel loop, and all blending is done via the gamma-corrected
//! [`ColorRGB::blend`] path to prevent dark-boundary artifacts.
//!
//! Generator function (alpha contribution per pixel):
//!
//!   alpha = 0.25 * (sin(d_u * x) + cos(d_v * y) + sin(d_w * x + rho) + cos(|z| * d_w4))
//!
//! where `z = (u, v)` is the normalised screen coordinate as a complex number.

use crate::{
    ColorRGB, Complex, ImageEffect, Monitor, NEON_PALETTES, NeonColor, RandomExt, WallSwitchResult,
    get_random_integer, process_rows_parallel_scoped,
};
use image::RgbImage;

// ============================================================================
// PARAMETER STRUCTS
// ============================================================================

/// Pre-computed frequency coefficients that are constant across the entire frame.
///
/// Separating them into a dedicated struct lets the compiler keep the values in
/// registers during the hot per-pixel inner loop.
pub struct AuroraParams {
    /// Horizontal sine frequency: `adjusted_density * 1.5 / width`.
    pub density_u: f64,
    /// Horizontal component of the diagonal-wave coefficient: `adjusted_density / width`.
    pub density_w_coeff: f64,
    /// Radial frequency: `adjusted_density * 1.2`.
    pub density_w4_coeff: f64,
    /// Reciprocal of the physical display width (avoids per-pixel division).
    pub inv_w: f64,
}

impl AuroraParams {
    /// Computes the aurora `alpha` contribution at horizontal pixel `x_f`.
    ///
    /// Uses [`Complex`] arithmetic for the radial term so the formula reads
    /// directly as the mathematical expression `|z| * d_w4`.
    #[inline(always)]
    pub fn alpha(&self, x_f: f64, row: &AuroraRowState) -> f64 {
        let u = x_f * self.inv_w;

        let w1 = (x_f * self.density_u).sin();
        let w3 = (x_f * self.density_w_coeff + row.v_density).sin();

        // Leverage Complex::abs() to compute the radial distance from the
        // normalised screen origin — identical to sqrt(u^2 + v^2) but expressed
        // directly in the complex-number algebra already used throughout the codebase.
        let screen_coord = Complex::new(u, row.v);
        let w4 = (screen_coord.abs() * self.density_w4_coeff).cos();

        let val = (w1 + row.w2 + w3 + w4) * 0.25;
        let wave = (val * std::f64::consts::PI).sin() * 0.5 + 0.5;
        let intensity = wave.powi(2);

        // Edge fade: uses Complex subtraction and abs() to measure distance from
        // the normalised viewport centre (0.5, 0.5), expressed as complex offset.
        let center_offset = (screen_coord - Complex::new(0.5, 0.5)) * 2.0;
        let edge_fade = (1.0 - center_offset.abs() * 0.45).clamp(0.0, 1.0);

        intensity * edge_fade * 0.65
    }
}

/// Dynamic state computed once per row and shared across all pixels in that row.
///
/// Hoisting these values out of the inner `x` loop avoids redundant trigonometric
/// evaluations that are independent of the horizontal coordinate.
pub struct AuroraRowState {
    /// Vertical coordinate normalised to `[0, 1]`.
    pub v: f64,
    /// `cos(v * density_v_coeff)` — the vertical cosine wave contribution.
    pub w2: f64,
    /// `v * density` — used in the diagonal sine wave `sin(x * d_w + v * d)`.
    pub v_density: f64,
    /// `v^2` — reserved for potential future radial terms.
    pub v_sq: f64,
}

// ============================================================================
// GENERATOR
// ============================================================================

/// A procedural generator for rendering wave-like Cosmic Aurora overlays.
pub struct AuroraGenerator {
    /// The base colour palette selected for the neon glow.
    pub color_palette: NeonColor,
    /// Base density frequency scaling factor.
    pub density: f64,
    /// Density multiplier tailored to the active aspect ratio.
    pub aspect_ratio_density_multiplier: f64,
}

impl ImageEffect for AuroraGenerator {
    fn apply(&self, rgb_img: &mut RgbImage) {
        let (width, height) = rgb_img.dimensions();
        let (w_f, h_f) = (width as f64, height as f64);

        let inv_w = 1.0 / w_f;
        let inv_h = 1.0 / h_f;
        let adjusted_density = self.density * self.aspect_ratio_density_multiplier;

        let params = AuroraParams {
            density_u: adjusted_density * 1.5 * inv_w,
            density_w_coeff: adjusted_density * inv_w,
            density_w4_coeff: adjusted_density * 1.2,
            inv_w,
        };

        let density_v_coeff = self.density * 2.0;
        let density_val = self.density;

        process_rows_parallel_scoped(rgb_img, |y, row_data| {
            let y_f = y as f64;
            let v = y_f * inv_h;

            let row_state = AuroraRowState {
                v,
                w2: (v * density_v_coeff).cos(),
                v_density: v * density_val,
                v_sq: v * v,
            };

            for (x, pixel_slice) in row_data.chunks_exact_mut(3).enumerate() {
                let alpha = params.alpha(x as f64, &row_state);

                if alpha > 0.01 {
                    let bg = ColorRGB::from_slice(pixel_slice);
                    // Gamma-corrected blend preserves perceived brightness.
                    bg.blend(self.color_palette.color_rgb, alpha)
                        .write_to_slice(pixel_slice);
                }
            }
        });
    }

    fn info(&self) -> String {
        format!(
            "overlay (density = {}), color: {}",
            self.density, self.color_palette
        )
    }
}

impl AuroraGenerator {
    /// Constructs a randomised, baseline Aurora configuration.
    ///
    /// This acts as the core constructor, randomly selecting a color palette
    /// and frequency density before any monitor adjustments are applied.
    pub fn new() -> WallSwitchResult<Self> {
        let color_palette = NEON_PALETTES.get_random_sample()?;
        let density = get_random_integer(4, 8);

        Ok(Self {
            color_palette,
            density,
            aspect_ratio_density_multiplier: 1.0,
        })
    }

    /// Generates a randomised Aurora configuration adjusted to the target monitor's aspect ratio.
    ///
    /// Reuses [`new`](Self::new) to initialize properties and calculates a density multiplier
    /// to preserve the structural aspect ratio of the aurora waves.
    pub fn random(monitor: &Monitor) -> WallSwitchResult<Self> {
        let mut aurora = Self::new()?;
        let width = monitor.resolution.width;
        let height = monitor.resolution.height;

        let aspect_ratio = width as f64 / height as f64;
        if aspect_ratio > 1.0 {
            aurora.aspect_ratio_density_multiplier = aspect_ratio.sqrt();
        }

        Ok(aurora)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests_aurora {
    use super::*;
    use crate::core::Monitor;
    use image::RgbImage;

    #[test]
    fn test_aurora_generator_new() -> WallSwitchResult<()> {
        let aurora = AuroraGenerator::new()?;
        assert!(
            aurora.density >= 4.0 && aurora.density <= 8.0,
            "density out of range: {}",
            aurora.density
        );
        assert_eq!(aurora.aspect_ratio_density_multiplier, 1.0);
        Ok(())
    }

    #[test]
    fn test_aurora_generator_random_density_in_range() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let aurora = AuroraGenerator::random(&monitor)?;
        assert!(
            aurora.density >= 4.0 && aurora.density <= 8.0,
            "density out of range: {}",
            aurora.density
        );
        Ok(())
    }

    #[test]
    fn test_aurora_applies_without_panic() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let aurora = AuroraGenerator::random(&monitor)?;
        let mut img = RgbImage::new(32, 18);
        aurora.apply(&mut img); // Must not panic
        Ok(())
    }

    #[test]
    fn test_aurora_params_alpha_range() {
        let params = AuroraParams {
            density_u: 0.05,
            density_w_coeff: 0.03,
            density_w4_coeff: 1.2,
            inv_w: 1.0 / 100.0,
        };
        let row = AuroraRowState {
            v: 0.5,
            w2: 0.0,
            v_density: 2.0,
            v_sq: 0.25,
        };
        for x in 0..100 {
            let alpha = params.alpha(x as f64, &row);
            assert!(
                (0.0..=1.0).contains(&alpha),
                "alpha out of range at x={x}: {alpha}"
            );
        }
    }

    #[test]
    fn test_aurora_aspect_ratio_multiplier() -> WallSwitchResult<()> {
        // Wide (landscape) monitor should produce multiplier > 1.
        let wide_monitor = Monitor {
            resolution: crate::Dimension {
                width: 3840,
                height: 1080,
            },
            ..Monitor::default()
        };
        let aurora = AuroraGenerator::random(&wide_monitor)?;
        assert!(
            aurora.aspect_ratio_density_multiplier > 1.0,
            "expected multiplier > 1 for wide monitor"
        );

        // Square monitor should produce multiplier == 1.
        let square_monitor = Monitor {
            resolution: crate::Dimension {
                width: 1080,
                height: 1080,
            },
            ..Monitor::default()
        };
        let aurora_sq = AuroraGenerator::random(&square_monitor)?;
        assert_eq!(aurora_sq.aspect_ratio_density_multiplier, 1.0);
        Ok(())
    }
}
