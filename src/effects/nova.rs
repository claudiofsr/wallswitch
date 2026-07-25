//! Nova Julia liquid fractal generator overlay.
//!
//! Generates flowing, fluid-like plumes resembling liquid mercury, cosmic
//! nebulae, or dynamic plasma current paths.
//!
//! Generator function:
//!
//! z(n+1) = z(n) - R * (z^p - 1) / (p * z^(p-1)) + c
//!
//! where p is the polynomial exponent, R is a complex relaxation modifier,
//! and c is a fixed perturbation coordinate.

use crate::{
    ColorRGB, Complex, Config, FractalConfig, FractalDescriptor, Monitor, NEON_PALETTES, RandomExt,
    RelaxedEscape, RelaxedViewportConfig, WallSwitchResult, get_random_integer,
    optimize_relaxed_viewport,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A named preset for the Nova Julia fractal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NovaPreset {
    /// Integer power of the polynomial f(z) = z^p - 1.
    pub power: u32,
    /// Julia additive perturbation constant (c).
    pub c: Complex,
    /// Complex relaxation factor (R).
    pub r: Complex,
    /// Human-readable name of the structural pattern.
    pub name: Cow<'static, str>,
}

const NOVA_ZOOM_RANGE: [f64; 2] = [1.2, 3.2];

const NOVA_PRESETS: &[NovaPreset] = &[
    NovaPreset {
        power: 3,
        c: Complex { re: 0.10, im: 0.15 },
        r: Complex { re: 1.00, im: 0.00 },
        name: Cow::Borrowed("Liquid Mercury Flow"),
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: -0.20,
            im: 0.45,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: Cow::Borrowed("Cosmic Plasma Flare"),
    },
    NovaPreset {
        power: 4,
        c: Complex { re: 0.22, im: 0.10 },
        r: Complex { re: 1.00, im: 0.00 },
        name: Cow::Borrowed("Ornate Coral Filigree"),
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: -0.35,
            im: 0.25,
        },
        r: Complex { re: 0.90, im: 0.00 },
        name: Cow::Borrowed("Nebulous Dust Whispers"),
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: -0.10,
            im: 0.35,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: Cow::Borrowed("Gilded Lace Tapestry"),
    },
    NovaPreset {
        power: 5,
        c: Complex {
            re: -0.05,
            im: 0.55,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: Cow::Borrowed("Glacial Frost Lattice"),
    },
];

/// A procedural generator for rendering Nova Julia liquid fractals.
pub struct NovaGenerator {
    /// Active polynomial, relaxation, and perturbation parameters.
    pub preset: NovaPreset,
    /// Shared viewport and rendering configuration.
    pub config: FractalConfig,
}

impl FractalDescriptor for NovaGenerator {
    #[inline(always)]
    fn config(&self) -> &FractalConfig {
        &self.config
    }

    #[inline(always)]
    fn center(&self) -> Complex {
        Complex::zero()
    }

    #[inline(always)]
    fn is_julia(&self) -> bool {
        true
    }

    #[inline(always)]
    fn render_pixel(&self, z_init: Complex, _scale: f64, max_radius: f64) -> (ColorRGB, f64, f64) {
        let (i, diff_norm, z_final) = nova_escape(
            z_init,
            self.preset.power,
            self.preset.r,
            self.preset.c,
            self.config.scan_iterations,
        );

        let edge_fade = z_init.circular_fade(max_radius, 0.40);
        RelaxedEscape {
            iterations: i,
            max_iterations: self.config.scan_iterations,
            diff_norm,
            z_final,
        }
        .color_nova(self.config.color_palette, edge_fade, (1e-5_f64).ln())
    }

    fn info_text(&self) -> String {
        format!(
            "fractal [{}]\n\
             f(z) = z^{} - 1 = 0, where c = {:5.2} {} {:4.2}i (iter = {:2}, zoom = {:.2}), color: {}",
            self.preset.name,
            self.preset.power,
            self.preset.c.re,
            if self.preset.c.im >= 0.0 { "+" } else { "-" },
            self.preset.c.im.abs(),
            self.config.scan_iterations,
            self.config.zoom,
            self.config.color_palette
        )
    }
}

impl NovaGenerator {
    /// Constructs a randomised, non-fitted Nova Julia generator.
    pub fn new(config: &Config) -> WallSwitchResult<Self> {
        let mut presets = Vec::new();

        if config.effects.add_presets {
            presets.extend(NOVA_PRESETS.iter().cloned());
        }

        for custom in &config.effects.nova {
            presets.push(NovaPreset {
                power: custom.power,
                c: custom.c,
                r: custom.r,
                name: Cow::Owned(custom.name.clone()),
            });
        }

        if presets.is_empty() {
            presets.extend(NOVA_PRESETS.iter().cloned());
        }

        let preset = presets.get_random_sample_cloned()?;
        let color_palette = NEON_PALETTES.get_random_sample()?;

        // Scale Nova search iterations safely relative to configured boundaries
        let scan_iterations = get_random_integer(
            config.effects.min_iterations.min(40),
            config.effects.max_iterations.clamp(41, 80),
        );

        Ok(Self {
            preset,
            config: FractalConfig {
                scan_iterations,
                color_palette,
                zoom: 1.8,
                rotation: Complex::sample_rotation(),
            },
        })
    }

    /// Constructs a randomised, monitor-fitted Nova Julia generator.
    pub fn random(monitor: &Monitor, config: &Config) -> WallSwitchResult<Self> {
        let mut nova = Self::new(config)?;
        nova.optimize_fit(monitor);
        Ok(nova)
    }

    /// Fits the viewport using the boundary-density sweep for Nova polynomials.
    pub fn optimize_fit(&mut self, monitor: &Monitor) {
        let (width, height) = (
            monitor.resolution.width as u32,
            monitor.resolution.height as u32,
        );
        let (power, r, c, iter) = (
            self.preset.power,
            self.preset.r,
            self.preset.c,
            self.config.scan_iterations,
        );

        let cfg = RelaxedViewportConfig {
            width,
            height,
            search_limit: 1.6,
            steps: 64,
            zoom_range: NOVA_ZOOM_RANGE,
            rand_range: [0.90, 1.35],
            fallback_range: [1.30, 2.80],
        };

        let (zoom, rotation) = optimize_relaxed_viewport(cfg, self.config.rotation, |z| {
            let (i, _, _) = nova_escape(z, power, r, c, iter);
            i > 6 && i < iter - 2
        });

        self.config.zoom = zoom;
        self.config.rotation = rotation;
    }
}

/// Evaluates the relaxed Nova Julia recurrence.
#[inline(always)]
pub fn nova_escape(
    z_init: Complex,
    power: u32,
    r: Complex,
    c: Complex,
    scan_iterations: u32,
) -> (u32, f64, Complex) {
    let mut z = z_init;
    let mut diff_norm = 1.0;

    for i in 0..scan_iterations {
        let z_norm_sq = z.abs_sq();
        if !(1e-6..=100.0).contains(&z_norm_sq) {
            return (i, diff_norm, z);
        }
        let step = r * z.newton_step_term(power);
        let z_next = z - step + c;
        diff_norm = (z_next - z).abs_sq();
        z = z_next;
        if diff_norm < 1e-5 {
            return (i, diff_norm, z);
        }
    }
    (scan_iterations, diff_norm, z)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests_nova {
    use super::*;

    #[test]
    fn test_nova_random_sanity() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let config = Config::default();
        let nova = NovaGenerator::random(&monitor, &config)?;
        assert!(nova.config.zoom > 0.0, "zoom must be positive");
        assert!(nova.preset.power > 0);
        assert!((nova.config.rotation.abs() - 1.0).abs() < 1e-9);
        Ok(())
    }

    #[test]
    fn test_nova_render_pixel_valid() -> WallSwitchResult<()> {
        let config = Config::default();
        let nova = NovaGenerator::new(&config)?;
        let (rgb, alpha, shadow) = nova.render_pixel(Complex::new(0.5, 0.5), 0.001, 3.0);
        for ch in rgb.to_array() {
            assert!((0.0..=1.0).contains(&ch), "channel out of range: {ch}");
        }
        assert!(alpha >= 0.0);
        assert!(shadow >= 0.0);
        Ok(())
    }
}
