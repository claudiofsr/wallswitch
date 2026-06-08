//! Newton-Raphson Basin of Attraction fractal generator overlay.
//!
//! Renders geometric, kaleidoscope-like mandala structures representing
//! root-finding convergence fields across complex space boundaries.
//!
//! Generator function:
//!
//! ```text
//! z(n+1) = z(n) - lambda * f(z) / f'(z),   f(z) = z^p - 1
//! ```

use crate::{
    ColorRGB, Complex, FractalConfig, FractalDescriptor, Monitor, NEON_PALETTES, RandomExt,
    RelaxedEscape, RelaxedViewportConfig, WallSwitchResult, get_random_integer,
    optimize_relaxed_viewport,
};

/// A named preset for the Newton-Raphson fractal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewtonPreset {
    /// Integer power of the polynomial `f(z) = z^p - 1`.
    pub power: u32,
    /// Complex relaxation factor (lambda).
    pub lambda: Complex,
    /// Human-readable name of the structural pattern.
    pub name: &'static str,
}

const NEWTON_ZOOM_RANGE: [f64; 2] = [1.5, 3.8];

const NEWTON_PRESETS: &[NewtonPreset] = &[
    NewtonPreset {
        power: 3,
        lambda: Complex { re: 1.00, im: 0.30 },
        name: "Gothic Rose Mandala",
    },
    NewtonPreset {
        power: 5,
        lambda: Complex { re: 0.90, im: 0.10 },
        name: "Imperial Star Compass",
    },
    NewtonPreset {
        power: 4,
        lambda: Complex { re: 1.00, im: 0.00 },
        name: "Stained Glass Kaleidoscope",
    },
    NewtonPreset {
        power: 6,
        lambda: Complex { re: 0.85, im: 0.20 },
        name: "Cosmic Snowflake Grid",
    },
    NewtonPreset {
        power: 3,
        lambda: Complex { re: 1.35, im: 0.00 },
        name: "Spiked Crown of Thorns",
    },
    NewtonPreset {
        power: 8,
        lambda: Complex { re: 0.70, im: 0.40 },
        name: "Quantum Energy Shells",
    },
    NewtonPreset {
        power: 5,
        lambda: Complex { re: 1.10, im: 0.25 },
        name: "Solar Flare Compass",
    },
    NewtonPreset {
        power: 3,
        lambda: Complex { re: 0.80, im: 0.50 },
        name: "Celtic Knotwork Ribbon",
    },
    NewtonPreset {
        power: 4,
        lambda: Complex { re: 0.60, im: 0.60 },
        name: "Nautilus Spiral Chamber",
    },
    NewtonPreset {
        power: 7,
        lambda: Complex { re: 1.00, im: 0.05 },
        name: "Hyper-Dimensional Matrix",
    },
    NewtonPreset {
        power: 6,
        lambda: Complex { re: 1.15, im: 0.15 },
        name: "Aetheric Frost Flower",
    },
    NewtonPreset {
        power: 8,
        lambda: Complex { re: 0.90, im: 0.30 },
        name: "Celestial Gearwork",
    },
    NewtonPreset {
        power: 3,
        lambda: Complex { re: 0.75, im: 0.60 },
        name: "Byzantine Dome",
    },
    NewtonPreset {
        power: 5,
        lambda: Complex {
            re: 1.25,
            im: -0.20,
        },
        name: "Abyssal Starfish",
    },
    NewtonPreset {
        power: 4,
        lambda: Complex { re: 0.80, im: 0.45 },
        name: "Hyperborean Sigil",
    },
    NewtonPreset {
        power: 7,
        lambda: Complex {
            re: 1.00,
            im: -0.30,
        },
        name: "Prismatic Labyrinth",
    },
    NewtonPreset {
        power: 3,
        lambda: Complex { re: 0.95, im: 0.80 },
        name: "Nebula Core Spiral",
    },
    NewtonPreset {
        power: 5,
        lambda: Complex { re: 0.60, im: 0.80 },
        name: "Aura Borealis Compass",
    },
    NewtonPreset {
        power: 10,
        lambda: Complex { re: 0.85, im: 0.00 },
        name: "Obsidian Glass Lattices",
    },
    NewtonPreset {
        power: 4,
        lambda: Complex {
            re: 1.40,
            im: -0.40,
        },
        name: "Bio-Polymer Filament",
    },
];

/// A procedural generator for rendering Newton-Raphson Basin fractals.
pub struct NewtonGenerator {
    /// Active polynomial and relaxation parameters.
    pub preset: NewtonPreset,
    /// Shared viewport and rendering configuration.
    pub config: FractalConfig,
}

impl FractalDescriptor for NewtonGenerator {
    #[inline(always)]
    fn config(&self) -> &FractalConfig {
        &self.config
    }

    /// Newton basins are centred at the complex origin.
    #[inline(always)]
    fn center(&self) -> Complex {
        Complex::zero()
    }

    /// Newton basins map the initial `z` across the viewport.
    #[inline(always)]
    fn is_julia(&self) -> bool {
        true
    }

    #[inline(always)]
    fn render_pixel(&self, z_init: Complex, _scale: f64, max_radius: f64) -> (ColorRGB, f64, f64) {
        let (i, diff_norm, z_final) = newton_escape(
            z_init,
            self.preset.power,
            self.preset.lambda,
            self.config.scan_iterations,
        );

        let edge_fade = z_init.circular_fade(max_radius, 0.40);
        RelaxedEscape {
            iterations: i,
            max_iterations: self.config.scan_iterations,
            diff_norm,
            z_final,
        }
        .color_newton(self.config.color_palette, edge_fade, (1e-6_f64).ln())
    }

    fn info_text(&self) -> String {
        format!(
            "fractal [{}]\n\
             f(z) = z^{} - 1 = 0, where l = {:5.2} {} {:4.2}i (iter = {:2}, zoom = {:.2}), color: {}",
            self.preset.name,
            self.preset.power,
            self.preset.lambda.re,
            if self.preset.lambda.im >= 0.0 {
                "+"
            } else {
                "-"
            },
            self.preset.lambda.im.abs(),
            self.config.scan_iterations,
            self.config.zoom,
            self.config.color_palette
        )
    }
}

impl NewtonGenerator {
    /// Constructs a randomised, non-fitted Newton-Raphson Basin generator.
    ///
    /// Selects a random coordinate preset, a random neon color palette,
    /// and a random rotation phasor. Returns an error if the preset or
    /// color palette slices are empty.
    pub fn new() -> WallSwitchResult<Self> {
        let preset = NEWTON_PRESETS.get_random_sample()?;
        let color_palette = NEON_PALETTES.get_random_sample()?;
        let scan_iterations = get_random_integer(40, 99);

        Ok(Self {
            preset,
            config: FractalConfig {
                scan_iterations,
                color_palette,
                zoom: 2.0,
                rotation: Complex::sample_rotation(),
            },
        })
    }

    /// Constructs a randomised, monitor-fitted Newton-Raphson Basin generator.
    ///
    /// Reuses [`new`](Self::new) to construct the base generator and then
    /// applies viewport optimization to match the target monitor's resolution.
    pub fn random(monitor: &Monitor) -> WallSwitchResult<Self> {
        let mut newton = Self::new()?;
        newton.optimize_fit(monitor);
        Ok(newton)
    }

    /// Fits the viewport using the boundary-density sweep for Newton polynomials.
    pub fn optimize_fit(&mut self, monitor: &Monitor) {
        let (width, height) = (
            monitor.resolution.width as u32,
            monitor.resolution.height as u32,
        );
        let (power, lambda, iter) = (
            self.preset.power,
            self.preset.lambda,
            self.config.scan_iterations,
        );

        let cfg = RelaxedViewportConfig {
            width,
            height,
            search_limit: 1.8,
            steps: 64,
            zoom_range: NEWTON_ZOOM_RANGE,
            rand_range: [0.95, 1.25],
            fallback_range: [1.50, 2.50],
        };

        let (zoom, rotation) = optimize_relaxed_viewport(cfg, self.config.rotation, |z| {
            let (i, _, _) = newton_escape(z, power, lambda, iter);
            i > 2 && i < iter - 2
        });

        self.config.zoom = zoom;
        self.config.rotation = rotation;
    }
}

/// Evaluates the relaxed Newton-Raphson recurrence on f(z) = z^p - 1.
///
/// Returns a tuple containing the final iteration count, the squared step norm,
/// and the final complex coordinate.
#[inline(always)]
pub fn newton_escape(
    z_init: Complex,
    power: u32,
    lambda: Complex,
    scan_iterations: u32,
) -> (u32, f64, Complex) {
    let mut z = z_init;
    let mut diff_norm = 1.0;

    for i in 0..scan_iterations {
        if z.abs_sq() < 1e-8 {
            return (i, diff_norm, z);
        }
        let step = lambda * z.newton_step_term(power);
        let z_next = z - step;
        diff_norm = step.abs_sq();
        z = z_next;
        if diff_norm < 1e-6 {
            return (i, diff_norm, z);
        }
    }
    (scan_iterations, diff_norm, z)
}

#[cfg(test)]
mod tests_newton {
    use super::*;

    #[test]
    fn test_newton_random_sanity() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let newton = NewtonGenerator::random(&monitor)?;
        assert!(newton.config.zoom > 0.0, "zoom must be positive");
        assert!(newton.config.scan_iterations > 0);
        assert!((newton.config.rotation.abs() - 1.0).abs() < 1e-9);
        Ok(())
    }

    #[test]
    fn test_newton_render_pixel_valid() {
        let newton = NewtonGenerator::new().unwrap();
        let (rgb, alpha, shadow) = newton.render_pixel(Complex::new(0.5, 0.5), 0.001, 3.0);
        for ch in rgb.to_array() {
            assert!((0.0..=1.0).contains(&ch), "channel out of range: {ch}");
        }
        assert!(alpha >= 0.0);
        assert!(shadow >= 0.0);
    }

    #[test]
    fn test_newton_escape_converges_on_root() {
        // The cube root 1+0i is a root of z^3 - 1 = 0.
        // Starting nearby should converge rapidly.
        let lambda = Complex::one();
        let (i, diff_norm, _) = newton_escape(Complex::new(1.01, 0.0), 3, lambda, 200);
        assert!(i < 200, "should converge before max_iter, got i={i}");
        assert!(
            diff_norm < 1e-5,
            "diff_norm not small at convergence: {diff_norm}"
        );
    }

    #[test]
    fn test_newton_escape_near_origin_returns_early() {
        // z very close to 0 -> |z|^2 < 1e-8 -> early return at i=0.
        let (i, _, _) = newton_escape(Complex::new(1e-5, 0.0), 3, Complex::one(), 200);
        assert_eq!(i, 0);
    }
}
