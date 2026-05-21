use crate::{WallSwitchError, WallSwitchResult, get_random_integer};
use clap::ValueEnum;
use image::RgbImage;
use serde::{Deserialize, Serialize};
use std::{io::Error, path::Path, thread};

/// Represents the supported procedural background overlay effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ProceduralEffect {
    #[value(name = "none")]
    #[default]
    None,
    #[value(name = "fractal")]
    JuliaFractal,
    #[value(name = "star")]
    Starfield,
    #[value(name = "random")]
    Random,
}

impl ProceduralEffect {
    /// Resolves the effect to a concrete rendering variant (resolving Random if selected).
    pub fn resolve(self) -> Self {
        match self {
            Self::Random => match get_random_integer(0, 1) {
                0 => Self::JuliaFractal,
                _ => Self::Starfield,
            },
            concrete => concrete,
        }
    }
}

/// Julia Fractal Generator and Image Post-Processor.
///
/// # Mathematical Concepts
///
/// ## 1. The Julia Set
/// The Julia set for a quadratic polynomial is defined by the behavior of the feedback loop:
/// `z_{n+1} = z_n^2 + c`
/// Where both `z` and `c` are complex numbers. For a given constant `c`, we iterate the function
/// starting at coordinates mapped to `z_0`. If the magnitude of `z` escapes to infinity (exceeds 2.0),
/// the pixel is colored based on the number of iterations it took to escape. If it remains bounded,
/// it belongs to the Julia set.
///
/// ## 2. Viewport Aspect Ratio Mapping and Random Zoom
/// To prevent the fractal from stretching or compressing on non-square screens (e.g., 16:9 or 21:9),
/// we map pixels to the complex plane using the minimum dimension (`min_dim`) as the scaling reference.
/// We also center the origin `(0,0)` at the exact midpoint of the image.
/// By applying a randomized zoom scale, we introduce varying levels of macro and micro-detail depth.
///
/// ## 3. Trigonometric 360-Degree Rotation
/// To rotate the fractal by a random angle `theta` around the origin without degrading performance,
/// we compute the rotation factors (sine and cosine) exactly once before entering the pixel loop:
/// `rx = cx * cos(theta) - cy * sin(theta)`
/// `ry = cx * sin(theta) + cy * cos(theta)`
///
/// ## 4. Continuous Potential (Smooth Coloring)
/// Integer iteration counts result in color banding artifacts (hard edges between color segments).
/// We solve this by applying a continuous potential calculation that factors in the escape magnitude:
/// `smooth_i = i + 1.0 - ln(ln(|z|)) / ln(2)`
/// This interpolates colors continuously, delivering smooth, professional-grade gradients.
///
/// ## 5. Contrast-Preserving Dynamic Halo Blending
/// To prevent the background image from losing its original contrast and intensity, we avoid flat
/// blending. Instead, blending factors are driven dynamically by the local fractal intensity `t`:
/// * When `t = 0` (outside the fractal), the background image is left completely untouched (100% intensity).
/// * When `t > 0` (inside fractal threads), a local shadow factor darkens the background directly beneath
///   the active pixels. This acts as a drop shadow, allowing vibrant neon fractal colors to stand out
///   clearly, even on pure white or busy backgrounds.
///
/// ## 6. Soft Vignette Shading
/// To seamlessly blend the edges and add depth, we apply a vignette filter.
/// The distance from the pixel to the center is calculated and mapped to a darkening multiplier,
/// creating a soft, professional edge fade.
pub struct FractalGenerator {
    /// Real component of the Julia set constant (c)
    pub c_re: f32,
    /// Imaginary component of the Julia set constant (c)
    pub c_im: f32,
    /// Maximum recursion iterations before determining convergence
    pub max_iterations: u32,
    /// RGB multipliers to tint the fractal intensity
    pub color_palette: [f32; 3],
    /// Randomized scale mapping determining the camera zoom level
    pub zoom: f32,
    /// Precalculated cosine of the random rotation angle
    pub cos_angle: f32,
    /// Precalculated sine of the random rotation angle
    pub sin_angle: f32,
}

impl Default for FractalGenerator {
    fn default() -> Self {
        Self {
            c_re: -0.7,
            c_im: 0.27015,
            max_iterations: 255,
            color_palette: [0.0, 1.0, 1.0], // Default Neon Cyan
            zoom: 3.0,
            cos_angle: 1.0, // 0 degrees rotation
            sin_angle: 0.0,
        }
    }
}

impl FractalGenerator {
    /// Creates a new customized Julia fractal generator.
    pub fn new(
        c_re: f32,
        c_im: f32,
        max_iterations: u32,
        color_palette: [f32; 3],
        zoom: f32,
        angle_degrees: f32,
    ) -> Self {
        let radians = angle_degrees.to_radians();
        Self {
            c_re,
            c_im,
            max_iterations,
            color_palette,
            zoom,
            cos_angle: radians.cos(),
            sin_angle: radians.sin(),
        }
    }

    /// Creates a FractalGenerator with a random aesthetic Julia constant,
    /// a randomized color palette, randomized zoom, and a randomized 360-degree rotation.
    pub fn random() -> Self {
        // Curated list of 34 mathematically interesting Julia constants (strictly highly detailed / non-filled)
        let presets = [
            (-0.7, 0.27015),     // 01. Classic dendrite
            (-0.4, 0.6),         // 02. Classic cloud swirls
            (-0.8, 0.156),       // 03. Detailed spirals
            (-0.7269, 0.1889),   // 04. Lace structures
            (-0.75, 0.11),       // 05. Feathery branches
            (-0.1, 0.651),       // 06. Cosmic dust style
            (-0.70176, -0.3842), // 07. Dragon-like curves (San Marco fractal boundary)
            (0.355, 0.355),      // 08. Spiral galaxy arms
            (-0.4, -0.59),       // 09. Swirling vortexes
            (-0.54, 0.54),       // 10. Ornamental lace borders
            (-0.74543, 0.11301), // 11. Dense filigree patterns
            (0.285, 0.535),      // 12. Twin spiral towers
            (-0.835, -0.2321),   // 13. Lightning rods
            (-0.77269, 0.12428), // 14. Coral reefs
            (-0.08, 0.72),       // 15. Twin spiral pinwheels
            (-0.51251, 0.5213),  // 16. Fine lace filaments
            (0.4, 0.4),          // 17. Symmetric stellar crowns (fine dust)
            (-0.55, 0.55),       // 18. Intricate leaf outlines
            (0.26, 0.0),         // 19. Parabolic needle-like valleys
            (-0.624, 0.435),     // 20. Crystalline snowflake patterns
            (-0.162, 1.04),      // 21. Towering minarets
            (-0.12, 0.85),       // 22. Flowing plasma plumes
            (-0.742, 0.1345),    // 23. Intricate branching nodes
            (-0.391, -0.587),    // 24. Swirling storm clouds
            (0.0, 0.8),          // 25. Classic symmetric dendrite (lightning branches)
            (-0.73, 0.21),       // 26. Feathery dendritic lace
            (-0.81, 0.2),        // 27. Spiral galaxy filaments
            (-0.68, 0.34),       // 28. Delicate coral spirals
            (-0.11, 0.83),       // 29. Plasma tendrils
            (-0.5, 0.56),        // 30. Intricate floral webs
            (-0.76, 0.08),       // 31. Lightning tree branches
            (-0.48, 0.53),       // 32. Swirling wave vortex
            (-0.72, 0.22),       // 33. Dendritic pine branches
            (-0.15, 0.75),       // 34. Detailed stellar pinwheels
        ];

        // High-contrast, vibrant neon color palettes to pop against normal images
        let palettes = [
            [1.0, 0.0, 0.8], // 01. Hot Pink / Cyberpunk Magenta
            [0.0, 1.0, 1.0], // 02. Electric Cyan
            [1.0, 0.6, 0.0], // 03. Vivid Orange / Gold
            [0.0, 1.0, 0.2], // 04. Laser Green
            [0.6, 0.0, 1.0], // 05. Deep Neon Violet
            [1.0, 0.1, 0.1], // 06. Vibrant Crimson Red
            [1.0, 1.0, 0.0], // 07. Radioactive Yellow
            [0.0, 0.4, 1.0], // 08. Cobalt Blue
            [0.5, 1.0, 0.0], // 09. Lime Glow
            [1.0, 0.0, 0.4], // 10. Electric Rose
            [0.0, 1.0, 0.6], // 11. Mint Neon
            [1.0, 0.4, 0.4], // 12. Soft Coral Flame
            [0.9, 0.9, 1.0], // 13. Bright Starlight White
            [1.0, 0.8, 0.0], // 14. Amber Glow
            [0.4, 0.0, 0.8], // 15. Electric Indigo
        ];

        let c_idx = get_random_integer(0, (presets.len() - 1) as u64) as usize;
        let p_idx = get_random_integer(0, (palettes.len() - 1) as u64) as usize;

        let zoom = get_random_integer(300, 400) as f32 / 100.0;
        let angle_degrees = get_random_integer(0, 359) as f32;
        let radians = angle_degrees.to_radians();

        let (c_re, c_im) = presets[c_idx];
        let color_palette = palettes[p_idx];

        Self {
            c_re,
            c_im,
            max_iterations: 255,
            color_palette,
            zoom,
            cos_angle: radians.cos(),
            sin_angle: radians.sin(),
        }
    }

    /// Applies the Julia fractal and vignette blending directly on an in-memory RgbImage in parallel.
    pub fn apply_effect_in_memory(&self, rgb_img: &mut RgbImage) {
        let (width, height) = rgb_img.dimensions();
        let w_f = width as f32;
        let h_f = height as f32;
        let min_dim = w_f.min(h_f);

        let scale = self.zoom / min_dim;
        let width_usize = width as usize;
        let row_stride = width_usize * 3;
        let pixels_buffer = rgb_img.as_mut();

        // Convert the flat buffer into mutable chunks representing distinct rows
        let mut rows: Vec<(usize, &mut [u8])> = pixels_buffer
            .chunks_exact_mut(row_stride)
            .enumerate()
            .collect();

        // Determine optimal thread partitioning
        let cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let chunk_size = (rows.len() / cores).max(1);

        thread::scope(|scope| {
            for chunk in rows.chunks_mut(chunk_size) {
                scope.spawn(move || {
                    for (y, row_data) in chunk.iter_mut() {
                        let y_f = *y as f32;

                        for x in 0..width_usize {
                            let x_f = x as f32;

                            // Map viewport scale based on the minimum dimension
                            let cx = (x_f - w_f / 2.0) * scale;
                            let cy = (y_f - h_f / 2.0) * scale;

                            // Apply 360-degree rotation around coordinates
                            let rx = cx * self.cos_angle - cy * self.sin_angle;
                            let ry = cx * self.sin_angle + cy * self.cos_angle;

                            let mut z_re = rx;
                            let mut z_im = ry;

                            let mut i = 0;
                            while i < self.max_iterations {
                                let re2 = z_re * z_re;
                                let im2 = z_im * z_im;
                                if re2 + im2 > 4.0 {
                                    break;
                                }
                                z_im = 2.0 * z_re * z_im + self.c_im;
                                z_re = re2 - im2 + self.c_re;
                                i += 1;
                            }

                            // Continuous potential smooth coloring formula
                            let t = if i < self.max_iterations {
                                let mag2 = z_re * z_re + z_im * z_im;
                                if mag2 > 4.0 {
                                    let log_zn = mag2.ln() / 2.0;
                                    let nu = (log_zn / 2.0_f32.ln()).ln() / 2.0_f32.ln();
                                    let smooth_i = (i as f32 + 1.0 - nu).max(0.0);
                                    (smooth_i / self.max_iterations as f32).clamp(0.0, 1.0)
                                } else {
                                    i as f32 / self.max_iterations as f32
                                }
                            } else {
                                1.0
                            };

                            let idx = x * 3;
                            let original_r = row_data[idx];
                            let original_g = row_data[idx + 1];
                            let original_b = row_data[idx + 2];

                            // Contrast-Preserving Dynamic Halo Blending (drop shadow under threads)
                            let shadow_factor = 1.0 - (t * 0.5);
                            let background_r = original_r as f32 * shadow_factor;
                            let background_g = original_g as f32 * shadow_factor;
                            let background_b = original_b as f32 * shadow_factor;

                            let r_fractal = self.color_palette[0] * t * 255.0;
                            let g_fractal = self.color_palette[1] * t * 255.0;
                            let b_fractal = self.color_palette[2] * t * 255.0;

                            let alpha = t.sqrt() * 0.8;
                            let blended_r = (background_r * (1.0 - alpha)) + (r_fractal * alpha);
                            let blended_g = (background_g * (1.0 - alpha)) + (g_fractal * alpha);
                            let blended_b = (background_b * (1.0 - alpha)) + (b_fractal * alpha);

                            // Soft Vignette Calculation
                            let dx = (x_f - w_f / 2.0) / (w_f / 2.0);
                            let dy = (y_f - h_f / 2.0) / (h_f / 2.0);
                            let dist = (dx * dx + dy * dy).sqrt();
                            let vignette = (1.0 - dist * 0.4).clamp(0.1, 1.0);

                            row_data[idx] = (blended_r * vignette).clamp(0.0, 255.0) as u8;
                            row_data[idx + 1] = (blended_g * vignette).clamp(0.0, 255.0) as u8;
                            row_data[idx + 2] = (blended_b * vignette).clamp(0.0, 255.0) as u8;
                        }
                    }
                });
            }
        });
    }

    /// Reads an input image, applies the Julia fractal in parallel, and saves to the output path.
    pub fn apply_effect<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> WallSwitchResult<()> {
        let img = image::open(&input_path)
            .map_err(|e| WallSwitchError::UnableToFind(format!("Failed to open image: {e}")))?;

        let mut rgb_img = img.to_rgb8();
        self.apply_effect_in_memory(&mut rgb_img);

        rgb_img
            .save(&output_path)
            .map_err(|e| WallSwitchError::Io(Error::other(e)))?;

        Ok(())
    }
}

// ==============================================================================
// ALTERNATIVE PROCEDURAL OVERLAYS
// ==============================================================================

/// Helper representation of individual stars in a Starfield.
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

/// Cyberpunk Starfield / Bokeh effect generator.
/// Projects randomized, floating, glowing circular points of light onto the canvas.
pub struct StarfieldGenerator {
    pub stars: Vec<Star>,
}

impl StarfieldGenerator {
    /// Generates a randomized list of glowing stars based on target monitor dimension limits.
    pub fn new(count: usize, width: u32, height: u32) -> Self {
        let mut stars = Vec::with_capacity(count);

        let palettes = [
            [1.0, 1.0, 1.0], // White
            [0.6, 0.8, 1.0], // Electric Ice Blue
            [1.0, 0.8, 0.4], // Cosmic Gold
            [1.0, 0.4, 0.8], // Ultraviolet Pink
        ];

        for _ in 0..count {
            let x = get_random_integer(0, width as u64) as f32;
            let y = get_random_integer(0, height as u64) as f32;
            let radius = get_random_integer(5, 45) as f32;
            let intensity = get_random_integer(30, 95) as f32 / 100.0;

            let p_idx = get_random_integer(0, (palettes.len() - 1) as u64) as usize;
            let color = palettes[p_idx];

            stars.push(Star {
                x,
                y,
                radius,
                color,
                intensity,
            });
        }

        Self { stars }
    }

    /// Appends smooth glowing star circles directly onto the image in parallel.
    pub fn apply_effect_in_memory(&self, rgb_img: &mut RgbImage) {
        let (width, _height) = rgb_img.dimensions();
        let width_usize = width as usize;
        let row_stride = width_usize * 3;
        let pixels_buffer = rgb_img.as_mut();

        let mut rows: Vec<(usize, &mut [u8])> = pixels_buffer
            .chunks_exact_mut(row_stride)
            .enumerate()
            .collect();

        let cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let chunk_size = (rows.len() / cores).max(1);

        thread::scope(|scope| {
            for chunk in rows.chunks_mut(chunk_size) {
                let stars = &self.stars;
                scope.spawn(move || {
                    for (y, row_data) in chunk.iter_mut() {
                        let y_f = *y as f32;

                        for x in 0..width_usize {
                            let x_f = x as f32;

                            let mut r_contrib = 0.0;
                            let mut g_contrib = 0.0;
                            let mut b_contrib = 0.0;
                            let mut total_alpha = 0.0;

                            // Calculate additive distance properties for local stars
                            for star in stars {
                                let dx = star.x - x_f;
                                let dy = star.y - y_f;
                                let dist_sq = dx * dx + dy * dy;

                                let star_radius_sq = star.radius * star.radius;
                                if dist_sq < star_radius_sq * 4.0 {
                                    // Gaussian intensity fallback matching physical light falloff
                                    let factor = (-dist_sq / (2.0 * star_radius_sq)).exp();
                                    let alpha = factor * star.intensity;

                                    r_contrib += star.color[0] * alpha;
                                    g_contrib += star.color[1] * alpha;
                                    b_contrib += star.color[2] * alpha;
                                    total_alpha += alpha;
                                }
                            }

                            if total_alpha > 0.001 {
                                let idx = x * 3;
                                let original_r = row_data[idx] as f32;
                                let original_g = row_data[idx + 1] as f32;
                                let original_b = row_data[idx + 2] as f32;

                                let alpha_clamp = total_alpha.min(0.95);

                                let blended_r = (original_r * (1.0 - alpha_clamp))
                                    + (r_contrib * 255.0 / total_alpha * alpha_clamp);
                                let blended_g = (original_g * (1.0 - alpha_clamp))
                                    + (g_contrib * 255.0 / total_alpha * alpha_clamp);
                                let blended_b = (original_b * (1.0 - alpha_clamp))
                                    + (b_contrib * 255.0 / total_alpha * alpha_clamp);

                                row_data[idx] = blended_r.clamp(0.0, 255.0) as u8;
                                row_data[idx + 1] = blended_g.clamp(0.0, 255.0) as u8;
                                row_data[idx + 2] = blended_b.clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                });
            }
        });
    }
}
