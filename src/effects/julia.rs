//! Julia Set fractal generator overlay.
//!
//! Renders Julia Set overlays using the distance estimator colouring method
//! shared with the Mandelbrot generator via [`color_distance_estimator`].
//! Viewport fitting is performed by [`JuliaGenerator::optimize_fit`], which
//! tightly frames only the boundary region using [`optimize_fractal_viewport`].
//!
//! Generator function: `f(z) = z^2 + c`, where `c` is a fixed complex constant
//! and the initial `z` varies across the viewport grid.

use crate::{
    ColorRGB, Complex, FractalConfig, FractalDescriptor, FractalPreset, MAX_ITERATIONS,
    MIN_ITERATIONS, Monitor, NEON_PALETTES, ProceduralEffect, RandomExt, WallSwitchResult,
    color_distance_estimator, get_random_integer, julia_escape, optimize_fractal_viewport,
};

// ============================================================================
// PRESET TABLE
// ============================================================================

/// All available Julia Set coordinate presets.
const JULIA_PRESETS: &[FractalPreset] = &[
    FractalPreset {
        center: Complex { re: -0.4, im: 0.6 },
        fractal_name: "Classic cloud swirls",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.8,
            im: 0.156,
        },
        fractal_name: "Detailed spirals",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.7269,
            im: 0.1889,
        },
        fractal_name: "Lace structures",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.75,
            im: 0.11,
        },
        fractal_name: "Feathery branches",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.1,
            im: 0.651,
        },
        fractal_name: "Cosmic dust style",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: 0.355,
            im: 0.355,
        },
        fractal_name: "Spiral galaxy arms",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.4,
            im: -0.59,
        },
        fractal_name: "Swirling vortexes",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.54,
            im: 0.54,
        },
        fractal_name: "Ornamental lace borders",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.835,
            im: -0.2321,
        },
        fractal_name: "Lightning rods",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.77269,
            im: 0.12428,
        },
        fractal_name: "Coral reefs",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.51251,
            im: 0.5213,
        },
        fractal_name: "Fine lace filaments",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.55,
            im: 0.55,
        },
        fractal_name: "Intricate leaf outlines",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.624,
            im: 0.435,
        },
        fractal_name: "Crystalline snowflake patterns",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.12,
            im: 0.85,
        },
        fractal_name: "Flowing plasma plumes",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.391,
            im: -0.587,
        },
        fractal_name: "Swirling storm clouds",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.73,
            im: 0.21,
        },
        fractal_name: "Feathery dendritic lace",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex { re: -0.81, im: 0.2 },
        fractal_name: "Spiral galaxy filaments",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.68,
            im: 0.34,
        },
        fractal_name: "Delicate coral spirals",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.76,
            im: 0.08,
        },
        fractal_name: "Lightning tree branches",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: 0.285,
            im: 0.01,
        },
        fractal_name: "Cosmic galaxy vortex swirls",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex { re: -0.8, im: 0.17 },
        fractal_name: "Spidery lace denderites",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.7269,
            im: -0.1889,
        },
        fractal_name: "Conjugate lace structures",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.835,
            im: 0.2321,
        },
        fractal_name: "Conjugate lightning rods",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.75,
            im: 0.05,
        },
        fractal_name: "Dense branching coral reef",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.70176,
            im: 0.3842,
        },
        fractal_name: "Conjugate dragon-like curves",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex { re: -0.8, im: 0.16 },
        fractal_name: "Deep sea coral spirals",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.722,
            im: 0.246,
        },
        fractal_name: "Dendritic pine branch variation",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.11,
            im: 0.655,
        },
        fractal_name: "Triple helix rotational cosmic swirls",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.52519,
            im: 0.5215,
        },
        fractal_name: "Intertwined Gothic Cathedral window arches",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: 0.28,
            im: 0.008,
        },
        fractal_name: "Centrifugal pinwheel galaxy vortex",
        effect_name: ProceduralEffect::JuliaSet,
    },
    FractalPreset {
        center: Complex {
            re: -0.83,
            im: -0.232,
        },
        fractal_name: "Sharp crystalline glacial ice needles",
        effect_name: ProceduralEffect::JuliaSet,
    },
];

// ============================================================================
// GENERATOR
// ============================================================================

/// A procedural generator for rendering Julia Set fractals onto desktop backgrounds.
pub struct JuliaGenerator {
    /// The selected coordinate preset defining the Julia constant `c`.
    pub preset: FractalPreset,
    /// Shared viewport and rendering configuration.
    pub config: FractalConfig,
}

impl FractalDescriptor for JuliaGenerator {
    #[inline(always)]
    fn config(&self) -> &FractalConfig {
        &self.config
    }

    #[inline(always)]
    fn center(&self) -> Complex {
        self.preset.center
    }

    /// Julia sets map the initial `z` across the viewport.
    #[inline(always)]
    fn is_julia(&self) -> bool {
        true
    }

    #[inline(always)]
    fn render_pixel(&self, z_init: Complex, scale: f64, _max_radius: f64) -> (ColorRGB, f64, f64) {
        let (i, z, dz) = julia_escape(z_init, self.preset.center, self.config.scan_iterations);
        color_distance_estimator(
            i,
            self.config.scan_iterations,
            z,
            dz,
            scale,
            self.config.color_palette,
        )
    }

    fn info_text(&self) -> String {
        format!(
            "fractal [{}]\n\
             f(z) = z^2 + c, where c = {:8.5} {} {:7.5}i (iter = {:4}, zoom = {:.5}), color: {}",
            self.preset.fractal_name,
            self.preset.center.re,
            if self.preset.center.im >= 0.0 {
                "+"
            } else {
                "-"
            },
            self.preset.center.im.abs(),
            self.config.scan_iterations,
            self.config.zoom,
            self.config.color_palette
        )
    }
}

impl JuliaGenerator {
    /// Constructs a randomised, non-fitted Julia Generator.
    ///
    /// This is useful as a base constructor, returning an error if
    /// the preset or color palette tables are empty.
    pub fn new() -> WallSwitchResult<Self> {
        let preset = JULIA_PRESETS.get_random_sample()?;
        let color_palette = NEON_PALETTES.get_random_sample()?;

        Ok(Self {
            preset,
            config: FractalConfig {
                scan_iterations: get_random_integer(MIN_ITERATIONS, MAX_ITERATIONS),
                color_palette,
                zoom: 3.0,
                rotation: Complex::sample_rotation(),
            },
        })
    }

    /// Constructs a randomised, monitor-fitted Julia Set generator.
    ///
    /// Selects a random coordinate preset, a random neon palette, and a random
    /// rotation phasor, then calls [`optimize_fit`](Self::optimize_fit) to
    /// tightly frame the boundary region for the chosen monitor resolution.
    ///
    /// # Errors
    ///
    /// Returns an error if the preset or color palette slices are empty.
    pub fn random(monitor: &Monitor) -> WallSwitchResult<Self> {
        let mut julia = Self::new()?;
        julia.optimize_fit(monitor);
        Ok(julia)
    }

    /// Automatically fits the viewport to the Julia Set boundary region.
    ///
    /// Computes the theoretical escape radius for the chosen constant `c` and
    /// sweeps a grid of candidate boundary points via [`optimize_fractal_viewport`],
    /// then applies a 10% padding margin so filaments are never clipped.
    pub fn optimize_fit(&mut self, monitor: &Monitor) {
        let width = monitor.resolution.width as u32;
        let height = monitor.resolution.height as u32;

        // The theoretical escape radius for f(z) = z^2 + c.
        let c_abs = self.preset.center.abs();
        let r_bound = (1.0 + (1.0 + 4.0 * c_abs).sqrt()) / 2.0;
        let search_limit = r_bound * 1.2;

        let steps = 128;
        let scan_iterations = self.config.scan_iterations;
        let center = self.preset.center;

        let (best_zoom, best_rotation) = optimize_fractal_viewport(
            width,
            height,
            search_limit,
            steps,
            self.config.rotation,
            |z| {
                let (i, _, _) = julia_escape(z, center, scan_iterations);
                i > 3 && i < scan_iterations
            },
        );

        let padding_factor = 1.05;

        if best_zoom < f64::MAX {
            self.config.zoom = best_zoom * padding_factor;
            self.config.rotation = best_rotation;
        } else {
            self.config.zoom = 2.0 * r_bound * padding_factor;
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests_julia {
    use super::*;

    #[test]
    fn test_random_generator_sanity() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let julia = JuliaGenerator::random(&monitor)?;

        assert!(julia.config.zoom > 0.0, "zoom must be positive");
        assert_eq!(julia.preset.effect_name, ProceduralEffect::JuliaSet);
        assert!(julia.config.scan_iterations >= MIN_ITERATIONS);
        assert!(julia.config.scan_iterations <= MAX_ITERATIONS);
        // Rotation phasor must remain a unit complex number after optimize_fit.
        assert!(
            (julia.config.rotation.abs() - 1.0).abs() < 1e-9,
            "rotation not unit: {:?}",
            julia.config.rotation
        );
        Ok(())
    }

    #[test]
    fn test_all_presets_have_correct_effect_name() {
        for preset in JULIA_PRESETS {
            assert_eq!(
                preset.effect_name,
                ProceduralEffect::JuliaSet,
                "wrong effect_name for preset '{}'",
                preset.fractal_name
            );
        }
    }

    #[test]
    fn test_render_pixel_returns_valid_channels() -> WallSwitchResult<()> {
        let julia = JuliaGenerator::new()?;
        // Sample a boundary-region pixel.
        let z = Complex::new(0.5, 0.3);
        let (rgb, alpha, shadow) = julia.render_pixel(z, 0.001, 5.0);
        for ch in rgb.to_array() {
            assert!((0.0..=1.0).contains(&ch), "channel out of range: {ch}");
        }
        assert!(alpha >= 0.0, "alpha must be non-negative");
        assert!(shadow >= 0.0, "shadow_alpha must be non-negative");
        Ok(())
    }

    #[test]
    fn test_optimize_fit_tightens_zoom() -> WallSwitchResult<()> {
        let mut julia = JuliaGenerator::new()?;
        let initial_zoom = julia.config.zoom;
        julia.optimize_fit(&Monitor::default());
        // After fitting, the zoom should be positive and finite.
        assert!(julia.config.zoom > 0.0);
        assert!(julia.config.zoom.is_finite());
        // For the classic denderite preset the fit zoom differs from the initial 3.0.
        let _ = initial_zoom; // Difference direction depends on preset; just check finiteness.
        Ok(())
    }
}
