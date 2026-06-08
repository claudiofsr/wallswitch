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
    ColorRGB, Complex, FractalConfig, FractalDescriptor, Monitor, NEON_PALETTES, RandomExt,
    RelaxedEscape, RelaxedViewportConfig, WallSwitchResult, get_random_integer,
    optimize_relaxed_viewport,
};

/// A named preset for the Nova Julia fractal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NovaPreset {
    /// Integer power of the polynomial f(z) = z^p - 1.
    pub power: u32,
    /// Julia additive perturbation constant (c).
    pub c: Complex,
    /// Complex relaxation factor (R).
    pub r: Complex,
    /// Human-readable name of the structural pattern.
    pub name: &'static str,
}

const NOVA_ZOOM_RANGE: [f64; 2] = [1.2, 3.2];

const NOVA_PRESETS: &[NovaPreset] = &[
    NovaPreset {
        power: 3,
        c: Complex { re: 0.10, im: 0.15 },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Liquid Mercury Flow",
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: -0.20,
            im: 0.45,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Cosmic Plasma Flare",
    },
    NovaPreset {
        power: 4,
        c: Complex { re: 0.22, im: 0.10 },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Ornate Coral Filigree",
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: -0.35,
            im: 0.25,
        },
        r: Complex { re: 0.90, im: 0.00 },
        name: "Nebulous Dust Whispers",
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: -0.10,
            im: 0.35,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Gilded Lace Tapestry",
    },
    NovaPreset {
        power: 5,
        c: Complex {
            re: -0.05,
            im: 0.55,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Glacial Frost Lattice",
    },
    NovaPreset {
        power: 3,
        c: Complex { re: 0.00, im: 0.12 },
        r: Complex { re: 1.15, im: 0.00 },
        name: "Spiritual Mandala Ripple",
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: 0.30,
            im: -0.20,
        },
        r: Complex { re: 0.80, im: 0.00 },
        name: "Bioluminescent Spore Nest",
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: 0.18,
            im: -0.40,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Abyssal Trench Vines",
    },
    NovaPreset {
        power: 6,
        c: Complex {
            re: -0.15,
            im: 0.15,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Hyperdimensional Loom",
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: -0.15,
            im: 0.35,
        },
        r: Complex { re: 1.00, im: 0.15 },
        name: "Gothic Cathedral Rose",
    },
    NovaPreset {
        power: 5,
        c: Complex { re: 0.25, im: 0.05 },
        r: Complex { re: 0.85, im: 0.25 },
        name: "Quantum Foam Fluctuation",
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: -0.28,
            im: -0.28,
        },
        r: Complex {
            re: 1.20,
            im: -0.10,
        },
        name: "Stellar Nucleosynthesis",
    },
    NovaPreset {
        power: 6,
        c: Complex { re: 0.05, im: 0.42 },
        r: Complex { re: 0.95, im: 0.05 },
        name: "Emerald Moss Labyrinth",
    },
    NovaPreset {
        power: 7,
        c: Complex {
            re: -0.08,
            im: 0.38,
        },
        r: Complex { re: 1.00, im: 0.00 },
        name: "Bismuth Crystal Citadel",
    },
    NovaPreset {
        power: 3,
        c: Complex { re: 0.32, im: 0.18 },
        r: Complex {
            re: 0.75,
            im: -0.30,
        },
        name: "Astral Jellyfish Canopy",
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: -0.45,
            im: 0.10,
        },
        r: Complex { re: 1.10, im: 0.15 },
        name: "Solar Prominence Loops",
    },
    NovaPreset {
        power: 5,
        c: Complex {
            re: -0.12,
            im: -0.32,
        },
        r: Complex {
            re: 0.90,
            im: -0.20,
        },
        name: "Aetheric Ley Line Matrix",
    },
    NovaPreset {
        power: 8,
        c: Complex { re: 0.15, im: 0.15 },
        r: Complex { re: 1.05, im: 0.10 },
        name: "Phytoplankton Radiance",
    },
    NovaPreset {
        power: 6,
        c: Complex {
            re: -0.22,
            im: 0.22,
        },
        r: Complex { re: 0.80, im: 0.40 },
        name: "Chronos Vortex Gear",
    },
    NovaPreset {
        power: 5,
        c: Complex {
            re: -0.18,
            im: 0.12,
        },
        r: Complex { re: 1.00, im: 0.30 },
        name: "Aeon Temple Portico",
    },
    NovaPreset {
        power: 8,
        c: Complex { re: 0.20, im: 0.35 },
        r: Complex {
            re: 0.90,
            im: -0.15,
        },
        name: "Hyperborean Crown",
    },
    NovaPreset {
        power: 3,
        c: Complex {
            re: -0.33,
            im: -0.05,
        },
        r: Complex { re: 1.10, im: 0.45 },
        name: "Abyssal Nautilus Shell",
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: 0.15,
            im: -0.55,
        },
        r: Complex { re: 0.70, im: 0.50 },
        name: "Spectral Dragon Spine",
    },
    NovaPreset {
        power: 3,
        c: Complex { re: 0.25, im: 0.25 },
        r: Complex {
            re: 1.30,
            im: -0.20,
        },
        name: "Opalescent Silk Ribbons",
    },
    NovaPreset {
        power: 5,
        c: Complex {
            re: -0.42,
            im: 0.18,
        },
        r: Complex {
            re: 1.00,
            im: -0.40,
        },
        name: "Phoenix Heart Nebula",
    },
    NovaPreset {
        power: 7,
        c: Complex {
            re: 0.30,
            im: -0.30,
        },
        r: Complex { re: 0.85, im: 0.10 },
        name: "Crystalline Geode Valley",
    },
    NovaPreset {
        power: 6,
        c: Complex {
            re: -0.02,
            im: 0.48,
        },
        r: Complex {
            re: 1.15,
            im: -0.30,
        },
        name: "Eldritch Eye Lattice",
    },
    NovaPreset {
        power: 4,
        c: Complex {
            re: -0.25,
            im: 0.30,
        },
        r: Complex { re: 0.90, im: 0.35 },
        name: "Prismatic Quantum Lattice",
    },
    NovaPreset {
        power: 9,
        c: Complex {
            re: 0.08,
            im: -0.28,
        },
        r: Complex { re: 1.00, im: 0.25 },
        name: "Void Weaver Spindle",
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

    /// Nova Julia fractals are centred at the complex origin.
    #[inline(always)]
    fn center(&self) -> Complex {
        Complex::zero()
    }

    /// Nova Julia maps the initial z across the viewport.
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
    ///
    /// This is useful as a base constructor, returning an error if
    /// the preset or color palette tables are empty.
    ///
    /// # Errors
    ///
    /// Returns [`WallSwitchError::EmptySlice`](crate::WallSwitchError::EmptySlice)
    /// if the preset or color palette slices are empty.
    pub fn new() -> WallSwitchResult<Self> {
        let preset = NOVA_PRESETS.get_random_sample()?;
        let color_palette = NEON_PALETTES.get_random_sample()?;
        let scan_iterations = get_random_integer(40, 80);

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
    ///
    /// Reuses [`new`](Self::new) to construct the base generator and then
    /// applies viewport optimization to match the monitor's aspect ratio.
    ///
    /// # Errors
    ///
    /// Returns [`WallSwitchError::EmptySlice`](crate::WallSwitchError::EmptySlice)
    /// if the preset or color palette slices are empty.
    pub fn random(monitor: &Monitor) -> WallSwitchResult<Self> {
        let mut nova = Self::new()?;
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
///
/// Returns `(iteration_count, squared_diff_norm, final_z)`.
///
/// The Newton step `(z^p - 1) / (p * z^(p-1))` is computed via
/// [`Complex::newton_step_term`]; the perturbation `c` is added at every step.
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
        let nova = NovaGenerator::random(&monitor)?;
        assert!(nova.config.zoom > 0.0, "zoom must be positive");
        assert!(nova.preset.power > 0);
        assert!((nova.config.rotation.abs() - 1.0).abs() < 1e-9);
        Ok(())
    }

    #[test]
    fn test_nova_render_pixel_valid() -> WallSwitchResult<()> {
        let nova = NovaGenerator::new()?;
        let (rgb, alpha, shadow) = nova.render_pixel(Complex::new(0.5, 0.5), 0.001, 3.0);
        for ch in rgb.to_array() {
            assert!((0.0..=1.0).contains(&ch), "channel out of range: {ch}");
        }
        assert!(alpha >= 0.0);
        assert!(shadow >= 0.0);
        Ok(())
    }
}
