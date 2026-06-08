//! Mandelbrot Set fractal generator overlay.
//!
//! Renders Mandelbrot Set overlays using the distance estimator colouring method
//! shared with the Julia generator via [`color_distance_estimator`]. The
//! `random` constructor runs a parallelised entropy sweep to find the zoom level
//! that maximises structural detail, then applies `dynamic_autofocus` to refine
//! the centre point and derive a level-of-detail iteration count.
//!
//! Generator function: `z(n+1) = z(n)^2 + c`, starting from `z = 0`,
//! where `c` varies over the viewport grid coordinates.

use crate::{
    ColorRGB, Complex, Config, FractalConfig, FractalDescriptor, FractalPreset, Monitor,
    NEON_PALETTES, ProceduralEffect, ROTATION_STEPS, RandomExt, Viewport, ViewportSpecs,
    WallSwitchResult, color_distance_estimator, get_random_integer, mandelbrot_escape,
};
use rayon::prelude::*;
use std::{borrow::Cow, cmp::Ordering};

/// All available hardcoded Mandelbrot coordinate presets.
const MANDELBROT_PRESETS: &[FractalPreset] = &[
    FractalPreset {
        center: Complex {
            re: -0.8115,
            im: 0.2014,
        },
        fractal_name: Cow::Borrowed("Tendril Valley Filaments"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.156,
            im: 1.033,
        },
        fractal_name: Cow::Borrowed("Dreadlock Valley Basin"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.38,
            im: 0.66,
        },
        fractal_name: Cow::Borrowed("Starburst Star Valley"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.56226,
            im: 0.64273,
        },
        fractal_name: Cow::Borrowed("Feathered Filament Cascades"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.77568377,
            im: 0.13646737,
        },
        fractal_name: Cow::Borrowed("Deep Seahorse Tail Spiral"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex { re: -1.45, im: 0.0 },
        fractal_name: Cow::Borrowed("West Needle Crown Filaments"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.55,
            im: 0.62,
        },
        fractal_name: Cow::Borrowed("Pentagonal Star Valley"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -1.625,
            im: 0.0,
        },
        fractal_name: Cow::Borrowed("Bi-Directional Filament"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.70176,
            im: 0.35422,
        },
        fractal_name: Cow::Borrowed("Peacock Tail Valley Plumes"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
    FractalPreset {
        center: Complex {
            re: -0.8113,
            im: 0.2015,
        },
        fractal_name: Cow::Borrowed("Medusa Tendril Clusters"),
        effect_name: ProceduralEffect::Mandelbrot,
    },
];

/// A procedural generator for rendering Mandelbrot Set fractals onto desktop backgrounds.
pub struct MandelbrotGenerator {
    /// The selected coordinate preset defining the viewport focus point.
    pub preset: FractalPreset,
    /// Shared viewport and rendering configuration.
    pub config: FractalConfig,
}

impl FractalDescriptor for MandelbrotGenerator {
    #[inline(always)]
    fn config(&self) -> &FractalConfig {
        &self.config
    }

    #[inline(always)]
    fn center(&self) -> Complex {
        self.preset.center
    }

    #[inline(always)]
    fn render_pixel(&self, c: Complex, scale: f64, _max_radius: f64) -> (ColorRGB, f64, f64) {
        let (i, z, dz) = mandelbrot_escape(c, self.config.scan_iterations);
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

impl MandelbrotGenerator {
    /// Constructs a randomized, non-fitted Mandelbrot Generator.
    pub fn new(config: &Config) -> WallSwitchResult<Self> {
        let mut presets = Vec::new();

        if config.effects.add_presets {
            presets.extend(MANDELBROT_PRESETS.iter().cloned());
        }

        for custom in &config.effects.mandelbrot {
            presets.push(FractalPreset {
                center: custom.center,
                fractal_name: Cow::Owned(custom.fractal_name.clone()),
                effect_name: ProceduralEffect::Mandelbrot,
            });
        }

        if presets.is_empty() {
            presets.extend(MANDELBROT_PRESETS.iter().cloned());
        }

        let preset = presets.get_random_sample_cloned()?;
        let color_palette = NEON_PALETTES.get_random_sample()?;

        Ok(Self {
            preset,
            config: FractalConfig {
                scan_iterations: config.effects.min_iterations,
                color_palette,
                zoom: 0.0025,
                rotation: Complex::one(),
            },
        })
    }

    /// Constructs a randomised, monitor-fitted Mandelbrot generator.
    pub fn random(monitor: &Monitor, config: &Config) -> WallSwitchResult<Self> {
        let (width, height) = (
            monitor.resolution.width as u32,
            monitor.resolution.height as u32,
        );

        let mut mandelbrot = Self::new(config)?;

        let rotation_phasors: Vec<Complex> = Complex::rotation_phasors(ROTATION_STEPS).collect();
        let zooms_count = get_random_integer(30, 50);
        let candidates = generate_zoom_candidates(zooms_count, ROTATION_STEPS);
        let preset_center = mandelbrot.preset.center;
        let min_iter = config.effects.min_iterations;

        let (best_base_zoom, best_rotation, _) = candidates
            .par_iter()
            .map(|&(base_zoom, r_idx)| {
                let aspect_ratio = width as f64 / height as f64;
                let adjusted_zoom = if aspect_ratio > 1.0 {
                    base_zoom * aspect_ratio.sqrt()
                } else {
                    base_zoom
                };

                let rotation = rotation_phasors[r_idx];
                let entropy = calculate_entropy(
                    preset_center,
                    adjusted_zoom,
                    rotation,
                    min_iter,
                    width,
                    height,
                );
                (base_zoom, rotation, entropy)
            })
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
            .unwrap_or((0.0002, Complex::one(), 0.0));

        mandelbrot.config.zoom = best_base_zoom;
        mandelbrot.config.rotation = best_rotation;

        mandelbrot.optimize_fit(width, height);
        mandelbrot.dynamic_autofocus(width, height, config);
        Ok(mandelbrot)
    }

    /// Scales the zoom for wide-aspect-ratio monitors.
    pub fn optimize_fit(&mut self, width: u32, height: u32) {
        let aspect_ratio = width as f64 / height as f64;
        if aspect_ratio > 1.0 {
            self.config.zoom *= aspect_ratio.sqrt();
        }
    }

    /// Refines the viewport centre and derives a level-of-detail iteration count.
    pub fn dynamic_autofocus(&mut self, width: u32, height: u32, config: &Config) {
        let search_radius = self.config.zoom * 0.25;
        let branch_phasor = find_branch_phasor(
            self.preset.center,
            search_radius,
            self.config.scan_iterations,
        );

        let aligned_center = locked_interior_grid_alignment(
            self.preset.center,
            branch_phasor,
            search_radius,
            self.config.scan_iterations,
        );
        self.preset.center = aligned_center;

        let best_entropy = calculate_entropy(
            self.preset.center,
            self.config.zoom,
            self.config.rotation,
            self.config.scan_iterations,
            width,
            height,
        );

        let climb_radius = self.config.zoom * 0.05;
        let search_directions: Vec<Complex> = std::iter::once(Complex::zero())
            .chain(Complex::rotation_phasors(ROTATION_STEPS).map(|p| p * climb_radius))
            .collect();

        let (best_center, _) = search_directions
            .par_iter()
            .map(|&offset| {
                let candidate = self.preset.center + offset;
                let entropy = calculate_entropy(
                    candidate,
                    self.config.zoom,
                    self.config.rotation,
                    self.config.scan_iterations,
                    width,
                    height,
                );
                (candidate, entropy)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .unwrap_or((self.preset.center, best_entropy));

        self.preset.center = best_center;

        let scale = self.config.zoom / (width.min(height) as f64);
        let lod = (150.0 + 45.0 * (1.0 / scale).ln()) as u32;
        self.config.scan_iterations =
            lod.clamp(config.effects.min_iterations, config.effects.max_iterations);
    }
}

// ============================================================================
// PRIVATE PURE HELPERS
// ============================================================================

fn find_branch_phasor(center: Complex, search_radius: f64, scan_iterations: u32) -> Complex {
    let mut best_phasor = Complex::one();
    let mut max_variation = -1.0_f64;

    for phasor in Complex::rotation_phasors(ROTATION_STEPS) {
        let mut total_variation = 0.0;
        let mut prev_i = 0;

        for k in 1..=4 {
            let sample = center + phasor * (search_radius * k as f64 * 0.25);
            let (i, _, _) = mandelbrot_escape(sample, scan_iterations);
            if k > 1 {
                total_variation += (i as f64 - prev_i as f64).abs();
            }
            prev_i = i;
        }

        if total_variation > max_variation {
            max_variation = total_variation;
            best_phasor = phasor;
        }
    }
    best_phasor
}

fn locked_interior_grid_alignment(
    center: Complex,
    phasor: Complex,
    search_radius: f64,
    scan_iterations: u32,
) -> Complex {
    const STEPS: usize = 64;
    let mut interior_segments: Vec<(usize, usize)> = Vec::new();
    let mut in_interior = false;
    let mut segment_start = 0;

    for step in 0..STEPS {
        let t = -search_radius + (step as f64 / (STEPS - 1) as f64) * (2.0 * search_radius);
        let (i, _, _) = mandelbrot_escape(center + phasor * t, scan_iterations);
        let is_interior = i >= scan_iterations;

        match (is_interior, in_interior) {
            (true, false) => {
                in_interior = true;
                segment_start = step;
            }
            (false, true) => {
                in_interior = false;
                interior_segments.push((segment_start, step - 1));
            }
            _ => {}
        }
    }
    if in_interior {
        interior_segments.push((segment_start, STEPS - 1));
    }

    let target = if interior_segments.len() >= 4 {
        Some(interior_segments[3])
    } else {
        interior_segments.last().copied()
    };

    if let Some((start_idx, end_idx)) = target {
        let mid_step = (start_idx + end_idx) as f64 / 2.0;
        let t_mid = -search_radius + (mid_step / (STEPS - 1) as f64) * (2.0 * search_radius);
        center + phasor * t_mid
    } else {
        center
    }
}

fn calculate_entropy(
    center: Complex,
    zoom: f64,
    rotation: Complex,
    scan_iterations: u32,
    width: u32,
    height: u32,
) -> f64 {
    const GRID: usize = 32;
    let mut histogram = vec![0u32; (scan_iterations + 1) as usize];

    let specs = ViewportSpecs {
        center,
        zoom,
        rotation,
        is_julia: false,
    };
    let viewport = Viewport::new(width as f64, height as f64, &specs);
    let (step_x, step_y) = (width as f64 / GRID as f64, height as f64 / GRID as f64);

    for gy in 0..GRID {
        let y_f = gy as f64 * step_y;
        for gx in 0..GRID {
            let (i, _, _) =
                mandelbrot_escape(viewport.map(gx as f64 * step_x, y_f), scan_iterations);
            histogram[i as usize] += 1;
        }
    }

    let total = (GRID * GRID) as f64;
    histogram.iter().filter(|&&c| c > 0).fold(0.0, |acc, &c| {
        let p = c as f64 / total;
        acc - p * p.ln()
    })
}

pub fn generate_zoom_candidates(zooms_count: usize, rotations_count: usize) -> Vec<(f64, usize)> {
    if zooms_count == 0 || rotations_count == 0 {
        return Vec::new();
    }

    const MIN_ZOOM: f64 = 2e-6;
    const MAX_ZOOM: f64 = 9.0;
    let log_ratio = MAX_ZOOM / MIN_ZOOM;

    let mut candidates = Vec::with_capacity(zooms_count * rotations_count);

    for z_idx in 0..zooms_count {
        let t = if zooms_count > 1 {
            z_idx as f64 / (zooms_count - 1) as f64
        } else {
            0.0
        };
        let zoom = MIN_ZOOM * log_ratio.powf(t);
        for r_idx in 0..rotations_count {
            candidates.push((zoom, r_idx));
        }
    }
    candidates
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests_mandelbrot {
    use super::*;

    #[test]
    fn test_mandelbrot_new_sanity() -> WallSwitchResult<()> {
        let config = Config::default();
        let m = MandelbrotGenerator::new(&config)?;
        assert!(m.config.zoom > 0.0, "zoom must be positive");
        assert_eq!(m.preset.effect_name, ProceduralEffect::Mandelbrot);
        Ok(())
    }

    #[test]
    fn test_random_generator_sanity() -> WallSwitchResult<()> {
        let monitor = Monitor::default();
        let config = Config::default();
        let m = MandelbrotGenerator::random(&monitor, &config)?;
        assert!(m.config.zoom > 0.0, "zoom must be positive");
        assert_eq!(m.preset.effect_name, ProceduralEffect::Mandelbrot);
        assert!(m.config.scan_iterations >= config.effects.min_iterations);
        assert!(m.config.scan_iterations <= config.effects.max_iterations);
        Ok(())
    }
}
