//! Procedural wallpaper overlay common utilities, math structures, and shared factory helpers.
//!
//! This module provides:
//! - Shared mathematical helpers (escape-time loops, viewport transforms, coloring functions).
//! - The [`ImageEffect`] trait, the blanket [`FractalDescriptor`] implementation, and the
//!   [`ProceduralEffect`] enum used across the entire effects sub-package.
//! - Parallel rendering infrastructure with built-in 2x2 Supersampling (SSAA)
//!   and 3D directional normal-vector embossing.

use crate::effects::{
    aurora::AuroraGenerator, julia::JuliaGenerator, mandelbrot::MandelbrotGenerator,
    newton::NewtonGenerator, nova::NovaGenerator, star::StarfieldGenerator,
};
use crate::{
    ColorRGB, Complex, Config, Monitor, NeonColor, WallSwitchError, WallSwitchResult,
    get_random_integer,
};
use clap::ValueEnum;
use image::RgbImage;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::FRAC_1_SQRT_2;
use std::{
    f64::consts::{LOG2_E, PI},
    io::Error,
    path::Path,
};

/// The number of angular steps used to evaluate structural rotations during optimization.
pub const ROTATION_STEPS: usize = 16;

// ============================================================================
// CORE TRAITS
// ============================================================================

/// Trait defining the behaviour for any image-processing overlay effect.
///
/// Implementations receive a mutable [`RgbImage`] buffer and blend their output
/// in-place, following the "Functional Core, Imperative Shell" pattern.
pub trait ImageEffect: Sync + Send {
    /// Applies the procedural effect in-place to a mutable image buffer.
    fn apply(&self, rgb_img: &mut RgbImage);

    /// Returns a formatted string containing diagnostic details of the active effect.
    fn info(&self) -> String;

    /// Convenience helper: opens `input_path`, runs the effect, writes `output_path`.
    fn apply_effect(&self, input_path: &Path, output_path: &Path) -> WallSwitchResult<()> {
        let img = image::open(input_path)
            .map_err(|e| WallSwitchError::UnableToFind(format!("Failed to open image: {e}")))?;

        let mut rgb_img = img.to_rgb8();
        self.apply(&mut rgb_img);

        rgb_img
            .save(output_path)
            .map_err(|e| WallSwitchError::Io(Error::other(e)))?;

        Ok(())
    }
}

/// Unified viewport layout and rendering configuration shared by all fractal generators.
///
/// Centralises the four parameters that every escape-time fractal needs so that
/// generators hold a single [`FractalConfig`] field instead of repeating them.
#[derive(Debug, Clone)]
pub struct FractalConfig {
    /// Maximum iteration limit for escape-time / convergence calculations.
    pub scan_iterations: u32,
    /// Base colour palette for neon glow blending.
    pub color_palette: NeonColor,
    /// Viewport zoom scale level (complex-plane units across the shorter screen axis).
    pub zoom: f64,
    /// Unit-phasor representing the viewport rotation angle (|rotation| == 1).
    pub rotation: Complex,
}

/// A polymorphic trait that defines the core algebraic structure for any procedural fractal.
///
/// Implementing this trait automatically provides an optimised [`ImageEffect`] implementation
/// via the blanket `impl` below, keeping the rendering engine fully decoupled from the
/// specific mathematics of each generator.
pub trait FractalDescriptor {
    /// Retrieves the shared viewport layout configuration.
    fn config(&self) -> &FractalConfig;

    /// Focus centre point on the complex plane.
    ///
    /// For Julia-family fractals the centre is the constant `c`; for Mandelbrot-family
    /// fractals it is the viewport centre used by [`Viewport`].
    fn center(&self) -> Complex;

    /// When `true` the viewport maps the initial `z`; when `false` it maps `c`.
    ///
    /// Defaults to `false` (Mandelbrot-style mapping).
    #[inline(always)]
    fn is_julia(&self) -> bool {
        false
    }

    /// Maps a pre-projected complex coordinate to its blended colour contribution.
    ///
    /// Returns `(rgb, alpha, shadow_alpha)`.
    fn render_pixel(&self, z_init: Complex, scale: f64, max_radius: f64) -> (ColorRGB, f64, f64);

    /// Returns a comprehensive diagnostic string formatted for the generator's equation.
    fn info_text(&self) -> String;
}

/// Blanket implementation: any type that implements [`FractalDescriptor`] automatically
/// gets a full, parallelised [`ImageEffect`] implementation — DRY by construction.
impl<T: FractalDescriptor + Sync + Send> ImageEffect for T {
    fn apply(&self, rgb_img: &mut RgbImage) {
        let cfg = self.config();
        render_fractal_parallel(
            rgb_img,
            cfg.zoom,
            cfg.rotation,
            self.center(),
            self.is_julia(),
            |z, scale, max_radius| self.render_pixel(z, scale, max_radius),
        );
    }

    fn info(&self) -> String {
        self.info_text()
    }
}

// ============================================================================
// PROCEDURAL EFFECT ENUM
// ============================================================================

/// Represents all supported procedural background overlay effects.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ProceduralEffect {
    /// No overlay effect is applied; displays the raw, unaltered wallpaper.
    #[value(name = "none")]
    #[default]
    None,

    /// Julia Set fractal overlay.
    ///
    /// * Characteristics: Rendered as thin, sharp, self-similar contour lines forming highly
    ///   symmetrical branching patterns. Depending on the selected complex constant, the lines trace
    ///   intricate shapes resembling swirling clouds, dendritic lace, spiral galaxy arms, leafy
    ///   filaments, or crystalline snowflakes.
    /// * Creator: Developed mathematically by the French mathematician Gaston Julia in 1918.
    /// * Generator function: Calculated by mapping the convergence boundary under the recursive function:
    ///   f(z) = z^2 + c
    ///   where c is a fixed complex constant perturbation and the initial coordinate z_0 varies across the viewport.
    #[value(name = "julia")]
    JuliaSet,

    /// Mandelbrot Set fractal overlay.
    ///
    /// * Characteristics: Rendered as thin, sharp, self-similar contour lines tracing the boundary of
    ///   the set. The lines expose highly detailed structural contours, including a main cardioid,
    ///   circular period bulbs, swirling spiral valleys, and repeating miniature copies of
    ///   the entire set connected by thin filaments.
    /// * Creator: First visualized and defined by the Polish-born French-American mathematician
    ///   Benoit Mandelbrot in 1980.
    /// * Generator function: Modeled using the quadratic recurrence equation starting from the origin:
    ///   z(n+1) = z(n)^2 + c
    ///   where z_0 = 0 and the complex parameter c varies across the viewport grid coordinates.
    #[value(name = "mandelbrot")]
    Mandelbrot,

    /// Newton-Raphson Basin of Attraction fractal overlay.
    ///
    /// * Characteristics: Symmetrical, kaleidoscope-like mandala structures representing root-finding
    ///   convergence fields across complex space boundaries. It maps the limits of convergence zones
    ///   where points migrate to specific roots of a polynomial equation.
    /// * Creator: Formulated based on Sir Isaac Newton's root-approximation methods (1690s) and Arthur
    ///   Cayley's subsequent complex-plane studies (1879).
    /// * Generator function: Computed using a relaxed Newton-Raphson recurrence formula:
    ///   z(n+1) = z(n) - lambda * f(z(n)) / f'(z(n))
    ///   on the polynomial f(z) = z^p - 1, where p is the integer polynomial power and lambda is a complex relaxation factor.
    #[value(name = "newton")]
    NewtonBasins,

    /// Nova Julia liquid fractal overlay.
    ///
    /// * Characteristics: Organic, flowing, fluid-like plumes resembling liquid mercury,
    ///   cosmic nebulae, or dynamic plasma current paths.
    /// * Creator: Developed by Paul Derbyshire in the late 1990s as a structural variation and
    ///   relaxation of the classic Newton-Raphson fractal.
    /// * Generator function: Evaluated using the relaxed Newton recurrence relation perturbed by a
    ///   dynamic additive complex value:
    ///   z(n+1) = z(n) - R * (z(n)^p - 1) / (p * z(n)^(p-1)) + c
    ///   where p is the polynomial exponent, R is a complex relaxation modifier, and c is a fixed perturbation coordinate.
    #[value(name = "nova")]
    NovaJulia,

    /// Procedural Cosmic Aurora wave generator.
    ///
    /// Generator function: multi-frequency sinusoidal wave composition.
    #[value(name = "aurora")]
    CosmicAurora,

    /// Procedural Starfield / Bokeh generator.
    ///
    /// Generator function: I(d) = I_0 * exp(-d^2 / (2 * sigma^2)) (Gaussian).
    #[value(name = "star")]
    Starfield,

    /// Fractal mode selector: randomly chooses between Julia or Mandelbrot.
    #[value(name = "fractal")]
    Fractal,

    /// Fractal mode selector: randomly chooses between Newton or Nova.
    #[value(name = "polynomial")]
    Polynomial,

    /// Fully randomised mode selector: picks any effect independently per display.
    #[value(name = "random")]
    Random,
}

impl ProceduralEffect {
    /// Human-readable display name for diagnostics and terminal output.
    pub fn get_name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::JuliaSet => "Julia Sets",
            Self::Mandelbrot => "Mandelbrot",
            Self::NewtonBasins => "Newton Basins",
            Self::NovaJulia => "Nova Julia",
            Self::CosmicAurora => "Cosmic Aurora",
            Self::Starfield => "Starfield",
            Self::Fractal => "Fractal",
            Self::Polynomial => "Polynomial",
            Self::Random => "Random",
        }
    }

    /// Resolves meta-variants (`Fractal`, `Random`) to a single concrete effect.
    ///
    /// Concrete variants pass through unchanged so callers can always call
    /// `resolve()` unconditionally.
    pub fn resolve(self) -> Self {
        match self {
            Self::Random => match get_random_integer(0, 5) {
                // Fractal
                0 => Self::JuliaSet,
                1 => Self::Mandelbrot,
                // Polynomial
                2 => Self::NewtonBasins,
                3 => Self::NovaJulia,
                // Others
                4 => Self::CosmicAurora,
                _ => Self::Starfield,
            },
            Self::Fractal => match get_random_integer(0, 1) {
                0 => Self::JuliaSet,
                _ => Self::Mandelbrot,
            },
            Self::Polynomial => match get_random_integer(0, 1) {
                0 => Self::NewtonBasins,
                _ => Self::NovaJulia,
            },
            concrete => concrete,
        }
    }

    /// Constructs a heap-allocated, monitor-fitted [`ImageEffect`] for this variant.
    ///
    /// Returns `None` for [`ProceduralEffect::None`] and the meta-variants
    /// `Fractal` / `Random` (callers should call [`resolve`](Self::resolve) first).
    ///
    /// # Errors
    ///
    /// Returns a [`WallSwitchError`] if the generator initialization fails.
    pub fn get_renderer(
        self,
        monitor: &Monitor,
        config: &Config,
    ) -> WallSwitchResult<Option<Box<dyn ImageEffect>>> {
        let renderer: Option<Box<dyn ImageEffect>> = match self {
            Self::JuliaSet => Some(Box::new(JuliaGenerator::random(monitor, config)?)),
            Self::Mandelbrot => Some(Box::new(MandelbrotGenerator::random(monitor, config)?)),
            Self::NewtonBasins => Some(Box::new(NewtonGenerator::random(monitor, config)?)),
            Self::NovaJulia => Some(Box::new(NovaGenerator::random(monitor, config)?)),
            Self::Starfield => Some(Box::new(StarfieldGenerator::random(monitor)?)),
            Self::CosmicAurora => Some(Box::new(AuroraGenerator::random(monitor)?)),
            _ => None,
        };

        Ok(renderer)
    }
}

// ============================================================================
// COORDINATE PRESET
// ============================================================================

/// A named complex-coordinate preset for escape-time fractal viewports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FractalPreset {
    /// Focal centre in the complex plane.
    pub center: Complex,
    /// Human-readable name of the structural pattern.
    pub fractal_name: std::borrow::Cow<'static, str>,
    /// Which effect category this preset belongs to.
    pub effect_name: ProceduralEffect,
}

impl std::fmt::Display for FractalPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:+.5} {:+.5}i) under {:?}",
            self.fractal_name, self.center.re, self.center.im, self.effect_name
        )
    }
}

// ============================================================================
// RELAXED-CONVERGENCE SHARED TYPES  (Newton / Nova)
// ============================================================================

/// Configuration options for fitting relaxed-convergence fractal viewports (Newton / Nova).
#[derive(Debug, Clone, Copy)]
pub struct RelaxedViewportConfig {
    /// Horizontal physical display resolution.
    pub width: u32,
    /// Vertical physical display resolution.
    pub height: u32,
    /// Maximum search boundary in complex coordinates.
    pub search_limit: f64,
    /// Number of steps in the coordinate search grid.
    pub steps: usize,
    /// Clamping zoom boundaries `[min, max]`.
    pub zoom_range: [f64; 2],
    /// Randomised fitting margin multipliers `[min, max]`.
    pub rand_range: [f64; 2],
    /// Fallback boundaries when the search sweep finds no boundary points `[min, max]`.
    pub fallback_range: [f64; 2],
}

/// Intermediate result of a relaxed Newton-Raphson or Nova Julia iteration sweep.
///
/// Carries just enough information for [`RelaxedEscape::color_newton`] and
/// [`RelaxedEscape::color_nova`] to produce their dual-tone convergence renders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelaxedEscape {
    /// Iteration count at convergence or bail-out.
    pub iterations: u32,
    /// Configured maximum iteration count.
    pub max_iterations: u32,
    /// Squared norm of the final Newton step (proximity-to-root metric).
    pub diff_norm: f64,
    /// Final complex coordinate at convergence.
    pub z_final: Complex,
}

impl RelaxedEscape {
    /// Computes a visually consistent dual-tone render for Newton-Raphson boundaries.
    ///
    /// Uses phase-based lighting and per-iteration colour cycling to create
    /// kaleidoscope-like mandala structures over the desktop background.
    #[inline(always)]
    pub fn color_newton(
        &self,
        color_palette: NeonColor,
        edge_fade: f64,
        ln_epsilon: f64,
    ) -> (ColorRGB, f64, f64) {
        self.color_impl(color_palette, edge_fade, ln_epsilon, false)
    }

    /// Computes a visually consistent dual-tone render for Nova Julia plumes.
    ///
    /// Uses higher-contrast wave profiles to enhance the fluid, organic character
    /// of Nova Julia structures compared to the Newton render.
    #[inline(always)]
    pub fn color_nova(
        &self,
        color_palette: NeonColor,
        edge_fade: f64,
        ln_epsilon: f64,
    ) -> (ColorRGB, f64, f64) {
        self.color_impl(color_palette, edge_fade, ln_epsilon, true)
    }

    /// Internal implementation shared by both Newton and Nova coloring paths.
    ///
    /// The `is_nova` flag selects slightly different wave exponents, glow weights,
    /// and shadow profiles tuned for each fractal's visual character.
    #[inline(always)]
    fn color_impl(
        &self,
        color_palette: NeonColor,
        edge_fade: f64,
        ln_epsilon: f64,
        is_nova: bool,
    ) -> (ColorRGB, f64, f64) {
        if self.iterations >= self.max_iterations {
            return (ColorRGB::default(), 0.0, 0.0);
        }

        let smooth_i = self.iterations as f64 + (self.diff_norm.ln() / ln_epsilon).clamp(0.0, 1.0);

        let ripple_frequency = 0.50_f64;
        let raw_wave = (smooth_i * ripple_frequency * PI).sin().abs();

        let norm_dist = if is_nova {
            raw_wave.powf(2.5)
        } else {
            raw_wave
        };

        // Core-line brightness and shadow profiles differ between Newton and Nova.
        let (core_thresh, core_range, glow_exp, glow_weight, shadow_exp) = if is_nova {
            (0.92_f64, 0.08_f64, 6_i32, 0.52_f64, 3_i32)
        } else {
            (0.95_f64, 0.05_f64, 5_i32, 0.40_f64, 2_i32)
        };

        let core = if norm_dist > core_thresh {
            (norm_dist - core_thresh) / core_range
        } else {
            0.0
        };
        let glow = norm_dist.powi(glow_exp) * glow_weight;
        let (profile_w_core, profile_w_glow) = if is_nova { (0.78, 0.22) } else { (0.70, 0.30) };
        let profile = core * profile_w_core + glow * profile_w_glow;
        let shadow_profile = (1.0 - norm_dist).powi(shadow_exp) * if is_nova { 0.48 } else { 0.35 };

        // Phase-based lighting: `arg(z_final)` maps the root sector to a shading angle.
        let angle = self.z_final.arg();
        let cos_arg = if is_nova {
            (angle * 4.0).cos().abs()
        } else {
            (angle * 3.0).cos().abs()
        };
        let light = if is_nova {
            0.75 + 0.25 * cos_arg
        } else {
            0.70 + 0.30 * cos_arg
        };

        let t_cycled = (smooth_i * 0.08) % 1.0;
        let secondary = color_palette.rotated_color();
        let core_color = if is_nova {
            let t_cos = (t_cycled * PI).cos() * 0.5 + 0.5;
            secondary.lerp(color_palette.color_rgb, t_cos)
        } else {
            secondary.lerp(color_palette.color_rgb, t_cycled)
        };

        let border_color = core_color.complementary().saturate_components();
        let blended = if is_nova {
            core_color.lerp(border_color, norm_dist.powf(3.0))
        } else {
            core_color.lerp(border_color, norm_dist)
        };

        let brightness_boost = if is_nova { 1.45 } else { 1.25 };
        let rgb = (blended * (light * brightness_boost)).clamp_bounds();

        let limit_fade_iter = if is_nova { 6 } else { 8 };
        let iteration_fade = if self.iterations < limit_fade_iter {
            self.iterations as f64 / limit_fade_iter as f64
        } else {
            1.0
        };

        (
            rgb,
            profile * 0.95 * iteration_fade * edge_fade,
            shadow_profile * iteration_fade * edge_fade,
        )
    }
}

// ============================================================================
// PARALLEL RENDERING INFRASTRUCTURE
// ============================================================================

/// Partitions an RGB image buffer into mutable row segments for thread-safe parallel processing.
pub fn partition_rows(rgb_img: &mut RgbImage) -> (Vec<(usize, &mut [u8])>, usize) {
    let (width, _) = rgb_img.dimensions();
    let row_stride = width as usize * 3;
    let rows: Vec<(usize, &mut [u8])> = rgb_img
        .as_mut()
        .chunks_exact_mut(row_stride)
        .enumerate()
        .collect();
    (rows, width as usize)
}

/// Executes row-by-row processing in parallel using Rayon's work-stealing thread pool.
///
/// The closure receives `(y: u32, row_data: &mut [u8])` where `row_data` is the
/// raw pixel bytes for the row (`width * 3` bytes, RGB packed).
pub fn process_rows_parallel_scoped<F>(rgb_img: &mut RgbImage, row_processor: F)
where
    F: Fn(u32, &mut [u8]) + Send + Sync,
{
    let (rows, _) = partition_rows(rgb_img);
    rows.into_par_iter()
        .for_each(|(y, row_data)| row_processor(y as u32, row_data));
}

/// Applies power-law (gamma) stretching to enhance the visual contrast of fractal filaments.
#[inline(always)]
pub fn stretch_potential(raw_t: f64) -> f64 {
    raw_t.clamp(0.0, 1.0).powf(0.35)
}

/// Calculates the continuous potential (smooth colouring) value for quadratic escape-time fractals.
///
/// Filters out low escape iterations to guarantee complete transparency in the far exterior.
#[inline]
pub fn calculate_smooth_potential(i: u32, max_iterations: u32, z: Complex) -> f64 {
    if i >= max_iterations {
        return 1.0;
    }

    let mag2 = z.abs_sq();
    let smooth_i = if mag2 > 4.0 {
        let log_zn = mag2.ln() * 0.5;
        let nu = log_zn.ln() * LOG2_E;
        (i as f64 + 1.0 - nu).max(0.0)
    } else {
        i as f64
    };

    let min_render_iter = 32.0_f64;
    if smooth_i < min_render_iter {
        return 0.0;
    }

    let normalized = (smooth_i - min_render_iter) / (max_iterations as f64 - min_render_iter);
    stretch_potential(normalized)
}

/// Calculates the analytical distance estimator (DEM) to the boundary of the fractal set.
#[inline]
pub fn calculate_distance_estimator(i: u32, max_iterations: u32, z: Complex, dz: Complex) -> f64 {
    if i < max_iterations {
        let z_mag = z.abs();
        let dz_mag = dz.abs();
        if z_mag > 0.0 && dz_mag > 0.0 {
            return 2.0 * z_mag * z_mag.ln() / dz_mag;
        }
    }
    0.0
}

/// Standard smoothstep interpolation function.
#[inline(always)]
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Blends the computed fractal colour and vignette shadow onto a mutable [`ColorRGB`] pixel.
///
/// Uses gamma-corrected blending via [`ColorRGB::blend`] for both the fractal
/// glow and the shadow darkening pass.
#[inline(always)]
pub fn blend_and_vignette(
    pixel: &mut ColorRGB,
    fractal_rgb: ColorRGB,
    alpha: f64,
    shadow_alpha: f64,
) {
    if shadow_alpha > 0.005 {
        *pixel = pixel.scale(1.0 - shadow_alpha);
    }
    if alpha > 0.005 {
        *pixel = pixel.blend(fractal_rgb, alpha);
    }
}

// ============================================================================
// HOT-LOOP ESCAPE-TIME EVALUATORS
// ============================================================================

/// Pure evaluation loop optimised for Julia Sets.
///
/// Iterates `z(n+1) = z(n)^2 + c` with simultaneous derivative tracking `dz`.
/// Returns `(iteration_count, final_z, final_dz)`.
#[inline(always)]
pub fn julia_escape(z_init: Complex, c: Complex, max_iter: u32) -> (u32, Complex, Complex) {
    let mut z = z_init;
    let mut dz = Complex::one();
    let mut i = 0;
    while i < max_iter && z.abs_sq() <= 4.0 {
        dz = dz * z * 2.0;
        z = z.square() + c;
        i += 1;
    }
    (i, z, dz)
}

/// Pure evaluation loop optimised for the Mandelbrot Set.
///
/// Incorporates early-exit shortcuts for the main cardioid and the period-2 bulb.
/// Returns `(iteration_count, final_z, final_dz)`.
#[inline(always)]
pub fn mandelbrot_escape(c: Complex, max_iter: u32) -> (u32, Complex, Complex) {
    // Cardioid / period-2 bulb shortcut — points inside escape immediately.
    let q = (c - Complex::new(0.25, 0.0)).abs_sq();
    if q * (q + (c.re - 0.25)) < 0.25 * c.im * c.im {
        return (max_iter, Complex::zero(), Complex::zero());
    }
    if (c + Complex::one()).abs_sq() < 0.0625 {
        return (max_iter, Complex::zero(), Complex::zero());
    }

    let mut z = Complex::zero();
    let mut dz = Complex::zero();
    let mut i = 0;
    while i < max_iter && z.abs_sq() <= 4.0 {
        dz = dz * z * 2.0 + Complex::one();
        z = z.square() + c;
        i += 1;
    }
    (i, z, dz)
}

// ============================================================================
// VIEWPORT MAPPING
// ============================================================================

/// Parameters used to construct a [`Viewport`] from physical screen dimensions.
pub struct ViewportSpecs {
    /// Focal complex centre point.
    pub center: Complex,
    /// Zoom translation scaling index.
    pub zoom: f64,
    /// Rotational coordinate transformation phasor.
    pub rotation: Complex,
    /// When `true`, the viewport maps around the origin (Julia-style).
    pub is_julia: bool,
}

/// Maps physical pixel coordinates `(x, y)` to complex plane coordinates via an
/// affine transform defined by `start`, `dx` (step per pixel in x) and `dy` (step
/// per pixel in y).
pub struct Viewport {
    /// Complex coordinate at pixel `(0, 0)`.
    pub start: Complex,
    /// Complex step per pixel along the screen X-axis.
    pub dx: Complex,
    /// Complex step per pixel along the screen Y-axis.
    pub dy: Complex,
}

impl Viewport {
    /// Constructs a viewport from physical screen size and rendering specs.
    pub fn new(width: f64, height: f64, specs: &ViewportSpecs) -> Self {
        let min_dim = width.min(height);
        let scale = specs.zoom / min_dim;

        let dx = specs.rotation * scale;
        let dy = dx * Complex::i();

        // Julia variants centre around the origin; Mandelbrot centres around specs.center.
        let v_center = if specs.is_julia {
            Complex::zero()
        } else {
            specs.center
        };
        let start = v_center - dx * (width / 2.0) - dy * (height / 2.0);

        Self { start, dx, dy }
    }

    /// Maps physical pixel coordinate `(x, y)` to the corresponding complex value.
    #[inline(always)]
    pub fn map(&self, x: f64, y: f64) -> Complex {
        self.start + self.dx * x + self.dy * y
    }
}

// ============================================================================
// COLOUR HELPERS FOR ESCAPE-TIME FRACTALS
// ============================================================================

/// Produces the distance-estimator colouring used by both Julia and Mandelbrot generators.
///
/// Combines a glow profile, smooth potential cycling, and 3-D directional normal embossing
/// into a single `(rgb, alpha, shadow_alpha)` triple ready for [`blend_and_vignette`].
#[inline(always)]
pub fn color_distance_estimator(
    i: u32,
    scan_iterations: u32,
    z: Complex,
    dz: Complex,
    scale: f64,
    color_palette: NeonColor,
) -> (ColorRGB, f64, f64) {
    let t = calculate_smooth_potential(i, scan_iterations, z);
    if t <= 0.005 || i >= scan_iterations {
        return (ColorRGB::default(), 0.0, 0.0);
    }

    let dist_complex = calculate_distance_estimator(i, scan_iterations, z, dz);
    let dist_pixels = dist_complex / scale;

    let max_radius = 4.0; // Slightly tighter glow radius to reduce blurriness (was 5.0)
    let shadow_radius = max_radius * 1.25; // Tighter shadow radius for sharper boundaries (was 1.5)

    if dist_pixels >= shadow_radius {
        return (ColorRGB::default(), 0.0, 0.0);
    }

    // Surface normal estimation for 3D embossing.
    // Approximated by the direction of the complex value z/dz.
    let u = z / dz;
    let u_abs = u.abs();
    let normal = if u_abs > 0.0 {
        u / u_abs
    } else {
        Complex::one()
    };

    // Standard directional light source (normalized vector pointing from top-left, i.e., -45 degrees)
    // Directional light source from top-left (135 degrees or 3 * pi/4 radians).
    // Pre-calculated using std constants to avoid hot-loop trig (sin/cos) overhead.
    let light_dir = Complex::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2);

    // Emboss dot product calculation between normal vector and light direction
    let dot = normal.re * light_dir.re + normal.im * light_dir.im;

    let norm_dist = (dist_pixels / max_radius).clamp(0.0, 1.0);

    // Highly defined, sharper core line
    let core = if dist_pixels < 1.0 {
        (1.0 - dist_pixels / 1.0).powi(2)
    } else {
        0.0
    };

    let ripple_freq = 12.0_f64;
    let ripple_wave = (t * PI * ripple_freq).sin().abs();
    let nested_detail =
        (1.0 - smoothstep(0.0, 0.4, 1.0 - ripple_wave)) * (1.0 - norm_dist).max(0.0);

    // Faster, tighter neon glow decay (using powi(8) instead of powi(6) to eliminate blurry halos)
    let glow = if dist_pixels < max_radius {
        (1.0 - norm_dist * norm_dist).powi(8) * 0.40
    } else {
        0.0
    };

    // Combine profile components with strong emphasis on the sharp core
    let profile = core * 0.70 + nested_detail * 0.15 + glow * 0.15;

    // Directional 3D Drop Shadow:
    // Casts a higher-contrast, asymmetric shadow opposite to the light direction
    let norm_shadow = (dist_pixels / shadow_radius).clamp(0.0, 1.0);
    let shadow_intensity = 0.65; // Highly pronounced shadow (was 0.35)
    let shadow_shading = (1.0 - dot * 0.45).clamp(0.1, 1.5); // Thicker shadow on opposite side
    let shadow_profile =
        (1.0 - norm_shadow * norm_shadow).powi(3) * shadow_intensity * shadow_shading;

    // Apply 3D emboss lighting to the fractal itself
    let light_emboss = 0.85 + 0.35 * dot; // Shading range of [0.5, 1.2]
    let light = light_emboss * (0.80 + 0.20 * (z.arg() * 4.0).cos().abs());
    let t_cycled = (t * 2.0) % 1.0;

    // Cycle between base palette and its channel-rotated secondary
    let secondary = color_palette.rotated_color();
    let core_color = if t_cycled < 0.5 {
        secondary.lerp(color_palette.color_rgb, t_cycled * 2.0)
    } else {
        color_palette
            .color_rgb
            .lerp(secondary, (t_cycled - 0.5) * 2.0)
    };

    let border_color = core_color.complementary().saturate_components();
    let blended = core_color.lerp(border_color, norm_dist.powi(2));
    let rgb = (blended * (light * 1.30)).clamp_bounds();

    let iteration_fade = if i < 16 { (i as f64 - 3.0) / 13.0 } else { 1.0 };

    (
        rgb,
        profile * 0.98 * iteration_fade,
        shadow_profile * iteration_fade,
    )
}

// ============================================================================
// VIEWPORT OPTIMISATION UTILITIES
// ============================================================================

/// Scans a complex coordinate grid and returns the optimal zoom and rotation phasor
/// that tightly frames all points for which `escape_check` returns `true`.
pub fn optimize_fractal_viewport<F>(
    width: u32,
    height: u32,
    search_limit: f64,
    steps: usize,
    rotation: Complex,
    mut escape_check: F,
) -> (f64, Complex)
where
    F: FnMut(Complex) -> bool,
{
    let inv_steps_minus_1 = 1.0 / (steps - 1) as f64;
    let range = 2.0 * search_limit;
    let mut active_points = Vec::with_capacity(steps * steps);

    for step_y in 0..steps {
        let ry = -search_limit + (step_y as f64 * inv_steps_minus_1) * range;
        for step_x in 0..steps {
            let rx = -search_limit + (step_x as f64 * inv_steps_minus_1) * range;
            let z = Complex::new(rx, ry);
            if escape_check(z) {
                active_points.push(z);
            }
        }
    }

    if !active_points.is_empty() {
        find_optimal_framing(&active_points, width, height, rotation)
    } else {
        (f64::MAX, rotation)
    }
}

/// Unified fitting helper for relaxed-convergence fractals (Newton / Nova).
///
/// Combines [`optimize_fractal_viewport`] with a random fitting margin and
/// automatic fallback to prevent degenerate zoom values.
pub fn optimize_relaxed_viewport<F>(
    config: RelaxedViewportConfig,
    rotation: Complex,
    mut escape_check: F,
) -> (f64, Complex)
where
    F: FnMut(Complex) -> bool,
{
    let (best_zoom, best_rotation) = optimize_fractal_viewport(
        config.width,
        config.height,
        config.search_limit,
        config.steps,
        rotation,
        &mut escape_check,
    );

    if best_zoom < f64::MAX {
        let rand_factor = get_random_integer::<_, f64>(
            (config.rand_range[0] * 100.0) as u64,
            (config.rand_range[1] * 100.0) as u64,
        ) / 100.0;
        let zoom = (best_zoom * rand_factor).clamp(config.zoom_range[0], config.zoom_range[1]);
        (zoom, best_rotation)
    } else {
        let flat_rand = get_random_integer::<_, f64>(
            (config.fallback_range[0] * 100.0) as u64,
            (config.fallback_range[1] * 100.0) as u64,
        ) / 100.0;
        (
            flat_rand.clamp(config.zoom_range[0], config.zoom_range[1]),
            rotation,
        )
    }
}

/// Core parallel fractal renderer with built-in 2x2 Supersampling Anti-Aliasing (SSAA).
///
/// Evaluates 4 sub-pixel samples per pixel and averages them in linear color space
/// to eliminate jagged edges and preserve microscopic fractal filaments.
pub fn render_fractal_parallel<F>(
    rgb_img: &mut RgbImage,
    zoom: f64,
    rotation: Complex,
    center: Complex,
    is_julia: bool,
    pixel_fn: F,
) where
    F: Fn(Complex, f64, f64) -> (ColorRGB, f64, f64) + Send + Sync,
{
    let (width, height) = rgb_img.dimensions();
    let (w_f, h_f) = (width as f64, height as f64);
    let aspect_ratio = w_f.max(h_f) / w_f.min(h_f);
    let max_radius = 0.98 * 0.5 * zoom * aspect_ratio;

    let specs = ViewportSpecs {
        center,
        zoom,
        rotation,
        is_julia,
    };
    let viewport = Viewport::new(w_f, h_f, &specs);
    let scale = zoom / w_f.min(h_f);

    // 2x2 sub-pixel offsets for grid supersampling
    let offsets = [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)];

    process_rows_parallel_scoped(rgb_img, |y, row_data| {
        let y_f = y as f64;
        for (x, pixel_slice) in row_data.chunks_exact_mut(3).enumerate() {
            let x_f = x as f64;

            let mut bg_color = ColorRGB::from_slice(pixel_slice);

            // Accumulators for linear-space supersampling
            let mut accumulated_fractal = ColorRGB::default();
            let mut accumulated_alpha = 0.0;
            let mut accumulated_shadow = 0.0;

            for &(ox, oy) in &offsets {
                let z_init = viewport.map(x_f + ox, y_f + oy);
                let (f_rgb, alpha, s_alpha) = pixel_fn(z_init, scale, max_radius);

                // Convert color to linear space before accumulation
                accumulated_fractal = accumulated_fractal + f_rgb.gamma2() * alpha;
                accumulated_alpha += alpha;
                accumulated_shadow += s_alpha;
            }

            // Average the 4 sub-pixel samples
            let alpha = accumulated_alpha * 0.25;
            let shadow_alpha = accumulated_shadow * 0.25;

            if alpha > 0.005 || shadow_alpha > 0.005 {
                let avg_fractal = if accumulated_alpha > 0.001 {
                    (accumulated_fractal * (1.0 / accumulated_alpha)).ungamma2()
                } else {
                    ColorRGB::default()
                };

                blend_and_vignette(&mut bg_color, avg_fractal, alpha, shadow_alpha);
                bg_color.write_to_slice(pixel_slice);
            }
        }
    });
}

/// Finds the optimal viewport rotation and zoom for a set of active complex points.
pub fn find_optimal_framing(
    active_points: &[Complex],
    width: u32,
    height: u32,
    default_rotation: Complex,
) -> (f64, Complex) {
    if active_points.is_empty() {
        return (f64::MAX, default_rotation);
    }

    let (w_f, h_f) = (width as f64, height as f64);
    let min_dim = w_f.min(h_f);

    let mut best_zoom = f64::MAX;
    let mut best_rotation = default_rotation;

    for phasor in Complex::rotation_phasors(ROTATION_STEPS) {
        let inverse_phasor = phasor.conj();
        let mut max_cx_abs = 0.0_f64;
        let mut max_cy_abs = 0.0_f64;

        for &point in active_points {
            let rotated = point * inverse_phasor;
            max_cx_abs = max_cx_abs.max(rotated.re.abs());
            max_cy_abs = max_cy_abs.max(rotated.im.abs());
        }

        let required_zoom =
            (2.0 * max_cx_abs * min_dim / w_f).max(2.0 * max_cy_abs * min_dim / h_f);

        if required_zoom < best_zoom {
            best_zoom = required_zoom;
            best_rotation = phasor;
        }
    }
    (best_zoom, best_rotation)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests_common {
    use crate::{NEON_PALETTES, RandomExt};

    use super::*;

    #[test]
    fn test_procedural_effect_resolution() {
        let resolved_rand = ProceduralEffect::Random.resolve();
        assert_ne!(resolved_rand, ProceduralEffect::Random);
        assert_ne!(resolved_rand, ProceduralEffect::Fractal);

        let resolved_fractal = ProceduralEffect::Fractal.resolve();
        assert_ne!(resolved_fractal, ProceduralEffect::Fractal);
        assert_ne!(resolved_fractal, ProceduralEffect::Random);

        // Concrete variants are identity under resolve().
        assert_eq!(
            ProceduralEffect::JuliaSet.resolve(),
            ProceduralEffect::JuliaSet
        );
        assert_eq!(ProceduralEffect::None.resolve(), ProceduralEffect::None);
    }

    #[test]
    fn test_smooth_potential_clamped() {
        let z = Complex::new(5.0, 5.0);
        let t = calculate_smooth_potential(50, 100, z);
        assert!((0.0..=1.0).contains(&t), "potential out of range: {t}");
    }

    #[test]
    fn test_smooth_potential_interior_returns_one() {
        // Points that never escape should return 1.0.
        let t = calculate_smooth_potential(100, 100, Complex::zero());
        assert_eq!(t, 1.0);
    }

    #[test]
    fn test_viewport_maps_center() {
        let specs = ViewportSpecs {
            center: Complex::new(0.5, -0.5),
            zoom: 2.0,
            rotation: Complex::one(),
            is_julia: false,
        };
        let viewport = Viewport::new(100.0, 100.0, &specs);
        let mapped = viewport.map(50.0, 50.0);
        assert!((mapped.re - 0.5).abs() < 1e-9, "re mismatch: {}", mapped.re);
    }

    #[test]
    fn test_rotation_phasors_are_unit() {
        let phasors: Vec<Complex> = Complex::rotation_phasors(ROTATION_STEPS).collect();
        assert_eq!(phasors.len(), ROTATION_STEPS);
        for p in phasors {
            assert!((p.abs() - 1.0).abs() < 1e-9, "phasor not unit: {p:?}");
        }
    }

    #[test]
    fn test_sample_helpers_in_range() -> WallSwitchResult<()> {
        for _ in 0..20 {
            let color_palette = NEON_PALETTES.get_random_sample()?;
            assert!(color_palette.color_rgb.red >= 0.0 && color_palette.color_rgb.red <= 1.0);

            let rot = Complex::sample_rotation();
            assert!((rot.abs() - 1.0).abs() < 1e-9, "rotation not unit: {rot:?}");
        }
        Ok(())
    }

    #[test]
    fn test_optimize_relaxed_viewport_in_bounds() {
        let cfg = RelaxedViewportConfig {
            width: 100,
            height: 100,
            search_limit: 1.5,
            steps: 10,
            zoom_range: [1.0, 3.0],
            rand_range: [0.9, 1.1],
            fallback_range: [1.2, 2.0],
        };
        let (zoom, rot) = optimize_relaxed_viewport(cfg, Complex::one(), |z| z.abs() < 1.0);
        assert!((1.0..=3.0).contains(&zoom), "zoom out of bounds: {zoom}");
        assert!((rot.abs() - 1.0).abs() < 1e-9, "rotation not unit: {rot:?}");
    }

    #[test]
    fn test_julia_escape_interior() {
        // c = 0 → z(n+1) = z(n)^2, starting at z=0 never escapes.
        let (i, _, _) = julia_escape(Complex::zero(), Complex::zero(), 100);
        assert_eq!(i, 100, "interior point should reach max_iter");
    }

    #[test]
    fn test_julia_escape_exterior() {
        // Starting outside the 2-radius bailout should escape on the first step.
        let (i, _, _) = julia_escape(Complex::new(3.0, 0.0), Complex::zero(), 100);
        assert_eq!(i, 0, "exterior point should escape immediately");
    }

    #[test]
    fn test_mandelbrot_escape_main_cardioid() {
        // The origin is the deepest interior point; it should reach max_iter.
        let (i, _, _) = mandelbrot_escape(Complex::zero(), 100);
        assert_eq!(i, 100);
    }

    #[test]
    fn test_relaxed_escape_color_consistency() {
        let palette = crate::NEON_PALETTES[0];

        let escape = RelaxedEscape {
            iterations: 10,
            max_iterations: 100,
            diff_norm: 1e-7,
            z_final: Complex::new(1.0, 0.0),
        };

        let (rgb_n, alpha_n, shadow_n) = escape.color_newton(palette, 1.0, (1e-6_f64).ln());
        let (rgb_v, alpha_v, shadow_v) = escape.color_nova(palette, 1.0, (1e-5_f64).ln());

        // Both should return valid, normalised values.
        for ch in rgb_n.to_array() {
            assert!(
                (0.0..=1.0).contains(&ch),
                "newton channel out of range: {ch}"
            );
        }
        for ch in rgb_v.to_array() {
            assert!((0.0..=1.0).contains(&ch), "nova channel out of range: {ch}");
        }
        assert!(alpha_n >= 0.0 && shadow_n >= 0.0);
        assert!(alpha_v >= 0.0 && shadow_v >= 0.0);
    }

    #[test]
    fn test_relaxed_escape_at_max_iter_returns_transparent() {
        let palette = crate::NEON_PALETTES[0];
        let escape = RelaxedEscape {
            iterations: 100,
            max_iterations: 100,
            diff_norm: 0.0,
            z_final: Complex::zero(),
        };
        let (_, alpha, shadow) = escape.color_newton(palette, 1.0, (1e-6_f64).ln());
        assert_eq!(alpha, 0.0);
        assert_eq!(shadow, 0.0);
    }
}
