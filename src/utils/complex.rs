use crate::rand;
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::PI,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// A complex number represented in Cartesian form by a real part (`re`) and an imaginary part (`im`).
///
/// In mathematics, a complex number z is defined as:
///
///   z = x + iy
///
/// where 'x' and 'y' are real numbers, and 'i' is the imaginary unit satisfying i^2 = -1.
/// The set of all complex numbers is denoted by C and forms a field under the standard operations
/// of addition and multiplication. Geometrically, C can be visualized as a two-dimensional
/// real vector space known as the complex plane (or Argand diagram), where the horizontal axis
/// represents the real component and the vertical axis represents the imaginary component.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    /// The real component of the complex number (x).
    pub re: f64,
    /// The imaginary component of the complex number (y).
    pub im: f64,
}

impl Complex {
    /// Creates a new complex number with the given real and imaginary parts.
    ///
    /// # Mathematical Description
    ///
    /// Maps a pair of real numbers (re, im) to a point in the complex plane:
    ///
    ///   z = re + i * im
    #[inline(always)]
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Returns the additive identity element of the complex field C.
    ///
    /// # Mathematical Description
    ///
    /// The complex number zero is defined as:
    ///
    ///   0 + 0i
    ///
    /// For any complex number z, it satisfies the additive identity property:
    ///
    ///   z + 0 = z
    #[inline(always)]
    pub const fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    /// Returns the multiplicative identity element of the complex field C.
    ///
    /// # Mathematical Description
    ///
    /// The complex number one is defined as:
    ///
    ///   1 + 0i
    ///
    /// For any complex number z, it satisfies the multiplicative identity property:
    ///
    ///   z * 1 = z
    #[inline(always)]
    pub const fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    /// Returns the imaginary unit i.
    ///
    /// # Mathematical Description
    ///
    /// The imaginary unit i is defined as:
    ///
    ///   i = 0 + 1i
    ///
    /// It satisfies the fundamental defining property:
    ///
    ///   i^2 = -1
    ///
    /// In the complex plane, multiplying a number by i corresponds to a
    /// counter-clockwise rotation of exactly 90 degrees (pi/2 radians).
    #[inline(always)]
    pub const fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    /// Returns the square of the complex number (z^2).
    ///
    /// # Mathematical Description
    ///
    /// Given z = x + iy, the square is derived by algebraic expansion:
    ///
    ///   z^2 = (x + iy)(x + iy) = x^2 + 2ixy + (iy)^2
    ///
    /// Since i^2 = -1, this simplifies to:
    ///
    ///   z^2 = (x^2 - y^2) + i(2xy)
    ///
    /// Calculating this directly avoids the general complex multiplication path
    /// (which requires 4 multiplications and 2 additions), reducing it to
    /// 3 multiplications and 1 subtraction, improving hot-loop execution performance.
    #[inline(always)]
    pub fn square(self) -> Self {
        Self {
            re: self.re * self.re - self.im * self.im,
            im: 2.0 * self.re * self.im,
        }
    }

    /// Returns the cube of the complex number (z^3).
    ///
    /// # Mathematical Description
    ///
    /// Given z = x + iy, the cube is evaluated by multiplying the squared value by z:
    ///
    ///   z^3 = z^2 * z = ((x^2 - y^2) + i(2xy)) * (x + iy)
    ///
    /// Unrolling this algebraic step reduces temporary memory allocations and maximizes
    /// instruction pipelining during high-iteration fractal sweeps.
    #[inline(always)]
    pub fn cube(self) -> Self {
        self.square() * self
    }

    /// Returns the multiplicative inverse (reciprocal) of the complex number (1/z).
    ///
    /// # Mathematical Description
    ///
    /// For any non-zero complex number z = x + iy, the inverse z^-1 is obtained by multiplying
    /// the numerator and denominator of 1/z by its complex conjugate z* = x - iy:
    ///
    ///   1/z = (x - iy) / ((x + iy)(x - iy)) = (x - iy) / (x^2 + y^2)
    ///
    /// This yields:
    ///
    ///   re = x / (x^2 + y^2)
    ///   im = -y / (x^2 + y^2)
    ///
    /// If z is zero (denominator is zero), this implementation returns 0 + 0i to prevent
    /// division-by-zero crashes.
    #[inline(always)]
    pub fn inv(self) -> Self {
        let denom = self.abs_sq();

        if denom == 0.0 {
            Self::zero()
        } else {
            Self {
                re: self.re / denom,
                im: -self.im / denom,
            }
        }
    }

    /// Raises the complex number to an unsigned integer power using binary exponentiation.
    ///
    /// # Mathematical Description
    ///
    /// Computes z^p. Instead of performing p - 1 linear multiplications, this method
    /// uses the binary representation of the exponent (exponentiation by squaring).
    ///
    /// By checking the least significant bit of the exponent and squaring the base at each step,
    /// the computational complexity is reduced from O(p) to O(log p).
    #[inline(always)]
    pub fn pow(self, p: u32) -> Self {
        let mut base = self;
        let mut exponent = p;
        let mut res = Self::one();
        while exponent > 0 {
            if exponent % 2 == 1 {
                res *= base;
            }
            base = base.square();
            exponent /= 2;
        }
        res
    }

    /// Raises the complex number to a signed integer power.
    ///
    /// # Mathematical Description
    ///
    /// Extends binary exponentiation to the set of all integers Z:
    ///
    /// - If exp > 0, computes z^exp.
    /// - If exp == 0, returns the multiplicative identity 1 + 0i (for any z).
    /// - If exp < 0, computes (1/z)^|exp| using the multiplicative inverse of z.
    #[inline(always)]
    pub fn powi(self, exp: i32) -> Self {
        if exp == 0 {
            return Self::one();
        }
        let mut base = if exp < 0 { self.inv() } else { self };
        let mut exponent = exp.unsigned_abs();
        let mut res = Self::one();
        while exponent > 0 {
            if exponent % 2 == 1 {
                res *= base;
            }
            base = base.square();
            exponent /= 2;
        }
        res
    }

    /// Raises the complex number to a real floating-point power (z^a).
    ///
    /// # Mathematical Description
    ///
    /// For a complex base z and a real exponent 'a', the power function is defined
    /// via the principal branch of the complex natural logarithm and exponential function:
    ///
    ///   z^a = exp(a * ln(z))
    ///
    /// If z is zero, the function returns zero (unless a == 0, which returns 1).
    #[inline(always)]
    pub fn powf(self, exp: f64) -> Self {
        if self.re == 0.0 && self.im == 0.0 {
            if exp == 0.0 {
                Self::one()
            } else {
                Self::zero()
            }
        } else {
            (self.ln() * exp).exp()
        }
    }

    /// Raises the complex number to a complex power (z^w).
    ///
    /// # Mathematical Description
    ///
    /// For a complex base z and a complex exponent w, the operation is defined as:
    ///
    ///   z^w = exp(w * ln(z))
    ///
    /// This definition uses the principal branch of the natural logarithm.
    #[inline(always)]
    pub fn powc(self, exp: Self) -> Self {
        if self.re == 0.0 && self.im == 0.0 {
            if exp.re == 0.0 && exp.im == 0.0 {
                Self::one()
            } else {
                Self::zero()
            }
        } else {
            (exp * self.ln()).exp()
        }
    }

    /// Computes the squared absolute value (squared norm) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The squared absolute value of z = x + iy is equal to the product of z and
    /// its complex conjugate z*:
    ///
    ///   |z|^2 = z * z* = (x + iy)(x - iy) = x^2 + y^2
    ///
    /// This represents the squared Euclidean distance from the origin in the complex plane.
    /// It is computationally faster than `abs()` because it avoids a square root operation.
    #[inline(always)]
    pub fn abs_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Calculates the absolute value (modulus or magnitude) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The absolute value of z = x + iy, denoted by |z|, is the Euclidean distance
    /// from the origin to the point (x, y) in the complex plane:
    ///
    ///   |z| = sqrt(x^2 + y^2)
    #[inline(always)]
    pub fn abs(self) -> f64 {
        self.abs_sq().sqrt()
    }

    /// Computes the squared Euclidean distance to another complex number.
    ///
    /// # Mathematical Description
    ///
    /// Given z1 = x1 + iy1 and z2 = x2 + iy2, the squared distance is:
    ///
    ///   d^2 = |z1 - z2|^2 = (x1 - x2)^2 + (y1 - y2)^2
    #[inline(always)]
    pub fn distance_sq(self, other: Self) -> f64 {
        (self - other).abs_sq()
    }

    /// Calculates the principal argument (angle) of the complex number in radians.
    ///
    /// # Mathematical Description
    ///
    /// The principal argument of z = x + iy, denoted by arg(z) or theta, is the angle
    /// formed between the positive real axis and the vector representing z.
    /// It is calculated using the two-argument arctangent function:
    ///
    ///   theta = atan2(y, x)
    ///
    /// The output value is bounded in the interval (-pi, pi].
    #[inline(always)]
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Calculates the complex conjugate of the complex number (z*).
    ///
    /// # Mathematical Description
    ///
    /// The complex conjugate of z = x + iy is denoted by z* (or z-bar) and is defined as:
    ///
    ///   z* = x - iy
    ///
    /// Geometrically, z* represents a reflection of z across the real (horizontal) axis.
    /// It satisfies several algebraic properties, such as:
    ///
    ///   z * z* = |z|^2
    ///   conj(z1 + z2) = conj(z1) + conj(z2)
    ///   conj(z1 * z2) = conj(z1) * conj(z2)
    #[inline(always)]
    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// Returns the signum of the complex number, which represents its normalized direction.
    ///
    /// # Mathematical Description
    ///
    /// The signum function sgn(z) maps a non-zero complex number z to the unit circle:
    ///
    ///   sgn(z) = z / |z|
    ///
    /// If z is zero, sgn(z) is defined as zero (0 + 0i).
    #[inline(always)]
    pub fn signum(self) -> Self {
        let mag = self.abs();

        if mag == 0.0 { Self::zero() } else { self / mag }
    }

    /// Creates a complex number from polar coordinates (radius and angle in radians).
    ///
    /// # Mathematical Description
    ///
    /// A complex number can be uniquely represented in polar form as:
    ///
    ///   z = r * (cos(theta) + i * sin(theta))
    ///
    /// where r >= 0 is the modulus (radius) and theta is the argument (angle).
    /// This conversion uses Euler's formula to map polar inputs to Cartesian coordinates.
    #[inline(always)]
    pub fn from_polar(radius: f64, angle: f64) -> Self {
        Self {
            re: radius * angle.cos(),
            im: radius * angle.sin(),
        }
    }

    /// Converts the complex number to polar coordinates (radius and angle in radians).
    ///
    /// Returns a tuple containing the modulus (radius) and the principal argument (angle): `(r, theta)`.
    ///
    /// # Mathematical Description
    ///
    /// Maps a Cartesian coordinate (x + iy) to its polar coordinate representation:
    ///
    ///   r = sqrt(x^2 + y^2)
    ///
    ///   theta = atan2(y, x)
    #[inline(always)]
    pub fn to_polar(self) -> (f64, f64) {
        (self.abs(), self.arg())
    }

    /// Computes Euler's formula for the given angle theta: e^(i * theta).
    ///
    /// # Mathematical Description
    ///
    /// Evaluates the exponential function of an imaginary input:
    ///
    ///   cis(theta) = e^(i * theta) = cos(theta) + i * sin(theta)
    ///
    /// Geometrically, this traces the unit circle centered at the origin in the complex plane.
    #[inline(always)]
    pub fn cis(theta: f64) -> Self {
        Self::from_polar(1.0, theta)
    }

    /// Rotates the complex number by a given complex phasor.
    ///
    /// # Mathematical Description
    ///
    /// Multiplication by a complex number w rotates and scales the original number.
    /// If w is a unit phasor (modulus |w| = 1), multiplication corresponds to a pure rotation:
    ///
    ///   arg(z * w) = arg(z) + arg(w)
    ///   |z * w| = |z|
    #[inline(always)]
    pub fn rotate(self, phasor: Self) -> Self {
        self * phasor
    }

    /// Computes the complex square root of the complex number.
    ///
    /// # Mathematical Derivation and Optimization
    ///
    /// Let z = x + iy be a non-zero complex number, and let r = |z| be its absolute value
    /// (computed via `self.abs()`). We seek a complex square root w = u + iv such that w^2 = z,
    /// which leads to the system of equations:
    ///
    ///   u^2 - v^2 = x
    ///   2uv = y
    ///
    /// Since u^2 + v^2 = r, we can solve for the components:
    ///
    ///   u = sqrt((r + x) / 2)
    ///   v = sign(y) * sqrt((r - x) / 2)
    ///
    /// To ensure numerical stability and prevent catastrophic cancellation (loss of significance)
    /// when x has a large magnitude, we define a common intermediate scaling factor:
    ///
    ///   t = sqrt((r + |x|) / 2)
    ///
    /// Because r and |x| are both positive, this calculation is always stable. We can then
    /// compute the components based on the sign of x:
    ///
    /// - If x >= 0:
    ///   u = t
    ///   v = y / (2 * t)
    ///
    /// - If x < 0:
    ///   u = |y| / (2 * t)
    ///   v = sign(y) * t
    ///
    /// By calculating the transcendental square root of 't' outside the conditional branches,
    /// we unify the instruction path, reduce compiler branching overhead, and ensure robust
    /// performance across all quadrants of the complex plane.
    #[inline(always)]
    pub fn sqrt(self) -> Self {
        if self.re == 0.0 && self.im == 0.0 {
            return Self::zero();
        }

        let r = self.abs();
        let t = (0.5 * (r + self.re.abs())).sqrt();

        let (re, im) = if self.re >= 0.0 {
            (t, self.im / (2.0 * t))
        } else {
            let re = self.im.abs() / (2.0 * t);
            let im = t.copysign(self.im);
            (re, im)
        };

        Self::new(re, im)
    }

    /// Computes the exponential function (e^z) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// For a complex number z = x + iy, the exponential function is defined as:
    ///
    ///   e^z = e^(x + iy) = e^x * e^(iy)
    ///
    /// Applying Euler's formula to the imaginary term yields:
    ///
    ///   e^z = e^x * (cos(y) + i * sin(y))
    ///
    /// Geometrically, the real part x determines the radial expansion factor e^x,
    /// while the imaginary part y acts as the rotational angle (argument) in radians.
    #[inline(always)]
    pub fn exp(self) -> Self {
        let r = self.re.exp();
        Self::new(r * self.im.cos(), r * self.im.sin())
    }

    /// Computes the principal branch of the natural logarithm of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The complex natural logarithm is a multi-valued function. Its principal branch,
    /// denoted by ln(z), is defined for z != 0 as:
    ///
    ///   ln(z) = ln(|z|) + i * arg(z)
    ///
    /// This maps the complex number to a horizontal strip in the complex plane, bounded
    /// vertically in the range (-pi, pi] along the imaginary axis.
    #[inline(always)]
    pub fn ln(self) -> Self {
        Self::new(self.abs().ln(), self.arg())
    }

    /// Computes the logarithm of the complex number with respect to an arbitrary real base.
    ///
    /// # Mathematical Description
    ///
    /// Using the change-of-base formula, the logarithm of z to the base 'b' is defined as:
    ///
    ///   log_b(z) = ln(z) / ln(b)
    #[inline(always)]
    pub fn log(self, base: f64) -> Self {
        self.ln() / base.ln()
    }

    /// Calculates the sine (sin) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The sine of a complex variable z = x + iy is defined using the complex exponential:
    ///
    ///   sin(z) = (e^(iz) - e^(-iz)) / (2i)
    ///
    /// Expanding this definition using Euler's formula yields:
    ///
    ///   sin(z) = sin(x) * cosh(y) + i * cos(x) * sinh(y)
    ///
    /// This relates the complex trigonometric functions directly to real trigonometric
    /// and hyperbolic functions.
    #[inline(always)]
    pub fn sin(self) -> Self {
        Self::new(
            self.re.sin() * self.im.cosh(),
            self.re.cos() * self.im.sinh(),
        )
    }

    /// Calculates the cosine (cos) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The cosine of z = x + iy is defined as:
    ///
    ///   cos(z) = (e^(iz) + e^(-iz)) / 2
    ///
    /// Expanding this definition yields:
    ///
    ///   cos(z) = cos(x) * cosh(y) - i * sin(x) * sinh(y)
    #[inline(always)]
    pub fn cos(self) -> Self {
        Self::new(
            self.re.cos() * self.im.cosh(),
            -self.re.sin() * self.im.sinh(),
        )
    }

    /// Calculates the tangent (tan) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The tangent of z is defined as the ratio of sine to cosine:
    ///
    ///   tan(z) = sin(z) / cos(z)
    #[inline(always)]
    pub fn tan(self) -> Self {
        self.sin() / self.cos()
    }

    /// Calculates the cosecant (csc) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The cosecant is the reciprocal of the sine function:
    ///
    ///   csc(z) = 1 / sin(z)
    #[inline(always)]
    pub fn csc(self) -> Self {
        Self::one() / self.sin()
    }

    /// Calculates the secant (sec) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The secant is the reciprocal of the cosine function:
    ///
    ///   sec(z) = 1 / cos(z)
    #[inline(always)]
    pub fn sec(self) -> Self {
        Self::one() / self.cos()
    }

    /// Calculates the cotangent (cot) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The cotangent is the reciprocal of the tangent function:
    ///
    ///   cot(z) = cos(z) / sin(z)
    #[inline(always)]
    pub fn cot(self) -> Self {
        self.cos() / self.sin()
    }

    /// Calculates the hyperbolic sine (sinh) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The hyperbolic sine of z = x + iy is defined as:
    ///
    ///   sinh(z) = (e^z - e^-z) / 2
    ///
    /// Expanding this definition yields:
    ///
    ///   sinh(z) = sinh(x) * cos(y) + i * cosh(x) * sin(y)
    #[inline(always)]
    pub fn sinh(self) -> Self {
        Self::new(
            self.re.sinh() * self.im.cos(),
            self.re.cosh() * self.im.sin(),
        )
    }

    /// Calculates the hyperbolic cosine (cosh) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The hyperbolic cosine of z = x + iy is defined as:
    ///
    ///   cosh(z) = (e^z + e^-z) / 2
    ///
    /// Expanding this definition yields:
    ///
    ///   cosh(z) = cosh(x) * cos(y) + i * sinh(x) * sin(y)
    #[inline(always)]
    pub fn cosh(self) -> Self {
        Self::new(
            self.re.cosh() * self.im.cos(),
            self.re.sinh() * self.im.sin(),
        )
    }

    /// Calculates the hyperbolic tangent (tanh) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The hyperbolic tangent is defined as the ratio of hyperbolic sine to hyperbolic cosine:
    ///
    ///   tanh(z) = sinh(z) / cosh(z)
    #[inline(always)]
    pub fn tanh(self) -> Self {
        self.sinh() / self.cosh()
    }

    /// Calculates the hyperbolic cosecant (csch) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The hyperbolic cosecant is the reciprocal of the hyperbolic sine:
    ///
    ///   csch(z) = 1 / sinh(z)
    #[inline(always)]
    pub fn csch(self) -> Self {
        Self::one() / self.sinh()
    }

    /// Calculates the hyperbolic secant (sech) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The hyperbolic secant is the reciprocal of the hyperbolic cosine:
    ///
    ///   sech(z) = 1 / cosh(z)
    #[inline(always)]
    pub fn sech(self) -> Self {
        Self::one() / self.cosh()
    }

    /// Calculates the hyperbolic cotangent (coth) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The hyperbolic cotangent is the reciprocal of the hyperbolic tangent:
    ///
    ///   coth(z) = cosh(z) / sinh(z)
    #[inline(always)]
    pub fn coth(self) -> Self {
        self.cosh() / self.sinh()
    }

    /// Calculates the inverse sine (arcsin) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The complex arcsine is defined using complex logarithms and square roots:
    ///
    ///   arcsin(z) = -i * ln(iz + sqrt(1 - z^2))
    ///
    /// It represents the inverse of the complex sine function.
    #[inline(always)]
    pub fn asin(self) -> Self {
        let i_z = Self::new(-self.im, self.re);
        let sqrt_term = (Self::one() - self.square()).sqrt();
        let ln_term = (i_z + sqrt_term).ln();
        Self::new(ln_term.im, -ln_term.re)
    }

    /// Calculates the inverse cosine (arccos) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The complex arccosine is computed using the geometric identity:
    ///
    ///   arccos(z) = pi/2 - arcsin(z)
    ///
    /// This identity is robust across all branches in C.
    #[inline(always)]
    pub fn acos(self) -> Self {
        Self::new(std::f64::consts::FRAC_PI_2, 0.0) - self.asin()
    }

    /// Calculates the inverse tangent (arctan) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The complex arctangent is defined logarithmic form as:
    ///
    ///   arctan(z) = i/2 * ln((i + z) / (i - z))
    #[inline(always)]
    pub fn atan(self) -> Self {
        let i = Self::i();
        let num = i + self;
        let den = i - self;
        let ln_term = (num / den).ln();
        Self::new(-0.5 * ln_term.im, 0.5 * ln_term.re)
    }

    /// Calculates the inverse hyperbolic sine (arcsinh) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The inverse hyperbolic sine is defined logarithmic form as:
    ///
    ///   arcsinh(z) = ln(z + sqrt(1 + z^2))
    #[inline(always)]
    pub fn asinh(self) -> Self {
        (self + (Self::one() + self.square()).sqrt()).ln()
    }

    /// Calculates the inverse hyperbolic cosine (arccosh) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The inverse hyperbolic cosine is defined logarithmic form as:
    ///
    ///   arccosh(z) = ln(z + sqrt(z^2 - 1))
    #[inline(always)]
    pub fn acosh(self) -> Self {
        (self + (self.square() - Self::one()).sqrt()).ln()
    }

    /// Calculates the inverse hyperbolic tangent (arctanh) of the complex number.
    ///
    /// # Mathematical Description
    ///
    /// The inverse hyperbolic tangent is defined logarithmic form as:
    ///
    ///   arctanh(z) = 1/2 * ln((1 + z) / (1 - z))
    #[inline(always)]
    pub fn atanh(self) -> Self {
        let num = Self::one() + self;
        let den = Self::one() - self;
        (num / den).ln() * 0.5
    }

    /// Evaluates the Newton-Raphson division term `(z^p - 1) / (p * z^(p-1))` for complex root finding.
    ///
    /// # Mathematical Description
    ///
    /// For a polynomial function f(z) = z^p - 1, its derivative with respect to z is
    /// f'(z) = p * z^(p-1). The Newton-Raphson step term represents the correction factor:
    ///
    ///   delta_z = f(z) / f'(z) = (z^p - 1) / (p * z^(p-1))
    ///
    /// Subtracting this term from z iteratively converges to one of the p-th roots of unity,
    /// forming the boundary mappings of Newton basins.
    #[inline(always)]
    pub fn newton_step_term(self, p: u32) -> Self {
        if p == 0 {
            return Self::zero();
        }
        let z_p_minus_1 = self.pow(p - 1);
        let f_z = z_p_minus_1 * self - Self::one();
        let f_prime_z = z_p_minus_1 * (p as f64);
        f_z / f_prime_z
    }

    /// Computes a flat-center circular edge-fade window (vignette/containment)
    /// with a smooth transition near the boundary radius.
    #[inline(always)]
    pub fn circular_fade(self, max_radius: f64, flat_ratio: f64) -> f64 {
        let dist = self.abs();
        let r = dist / max_radius;

        if r < flat_ratio {
            1.0
        } else if r < 1.0 {
            // Smooth transition from 1.0 down to 0.0 using smoothstep
            let t = (r - flat_ratio) / (1.0 - flat_ratio);
            1.0 - t * t * (3.0 - 2.0 * t)
        } else {
            0.0
        }
    }

    /// Generates a uniformly random unit-phasor `e^(i*theta)` with `theta` in `[0, 2*pi)`.
    ///
    /// This is an associated function that constructs a new `Complex` number on the unit circle
    /// with a random angle, useful for phase rotations and randomized fractal seeding.
    #[inline]
    pub fn sample_rotation() -> Self {
        let radians = rand::<f64>() * 2.0 * PI;
        Self::cis(radians)
    }

    /// Returns an iterator over unit complex phasors evenly distributed around the unit circle.
    #[inline]
    pub fn rotation_phasors(rotations: usize) -> impl Iterator<Item = Self> {
        (0..rotations).map(move |step| Self::cis((step as f64) * 2.0 * PI / (rotations as f64)))
    }
}

// ==========================================
// OPERATOR OVERLOADS
// ==========================================

impl Add for Complex {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.re + other.re, self.im + other.im)
    }
}

impl Add<f64> for Complex {
    type Output = Self;

    #[inline(always)]
    fn add(self, scalar: f64) -> Self {
        Self::new(self.re + scalar, self.im)
    }
}

impl Add<Complex> for f64 {
    type Output = Complex;

    #[inline(always)]
    fn add(self, other: Complex) -> Complex {
        Complex::new(self + other.re, other.im)
    }
}

impl Sub for Complex {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.re - other.re, self.im - other.im)
    }
}

impl Sub<f64> for Complex {
    type Output = Self;

    #[inline(always)]
    fn sub(self, scalar: f64) -> Self {
        Self::new(self.re - scalar, self.im)
    }
}

impl Sub<Complex> for f64 {
    type Output = Complex;

    #[inline(always)]
    fn sub(self, other: Complex) -> Complex {
        Complex::new(self - other.re, -other.im)
    }
}

impl Mul for Complex {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Self) -> Self::Output {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

impl Div for Complex {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Self) -> Self::Output {
        let denom = other.abs_sq();
        if denom == 0.0 {
            Self::new(0.0, 0.0)
        } else {
            Self::new(
                (self.re * other.re + self.im * other.im) / denom,
                (self.im * other.re - self.re * other.im) / denom,
            )
        }
    }
}

impl Neg for Complex {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

impl Mul<f64> for Complex {
    type Output = Self;
    #[inline(always)]
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.re * scalar, self.im * scalar)
    }
}

impl Mul<Complex> for f64 {
    type Output = Complex;
    #[inline(always)]
    fn mul(self, other: Complex) -> Complex {
        Complex::new(self * other.re, self * other.im)
    }
}

impl Div<f64> for Complex {
    type Output = Self;
    #[inline(always)]
    fn div(self, scalar: f64) -> Self {
        if scalar == 0.0 {
            Self::new(0.0, 0.0)
        } else {
            Self::new(self.re / scalar, self.im / scalar)
        }
    }
}

impl Div<Complex> for f64 {
    type Output = Complex;
    #[inline(always)]
    fn div(self, other: Complex) -> Complex {
        let denom = other.abs_sq();
        if denom == 0.0 {
            Complex::new(0.0, 0.0)
        } else {
            Complex::new((self * other.re) / denom, (-self * other.im) / denom)
        }
    }
}

// Assignment Operators

impl AddAssign for Complex {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl AddAssign<f64> for Complex {
    #[inline(always)]
    fn add_assign(&mut self, other: f64) {
        self.re += other;
    }
}

impl SubAssign for Complex {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        self.re -= other.re;
        self.im -= other.im;
    }
}

impl SubAssign<f64> for Complex {
    #[inline(always)]
    fn sub_assign(&mut self, other: f64) {
        self.re -= other;
    }
}

impl MulAssign for Complex {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl MulAssign<f64> for Complex {
    #[inline(always)]
    fn mul_assign(&mut self, other: f64) {
        self.re *= other;
        self.im *= other;
    }
}

impl DivAssign for Complex {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl DivAssign<f64> for Complex {
    #[inline(always)]
    fn div_assign(&mut self, other: f64) {
        if other == 0.0 {
            self.re = 0.0;
            self.im = 0.0;
        } else {
            self.re /= other;
            self.im /= other;
        }
    }
}

#[cfg(test)]
mod tests_complex {
    use super::*;

    fn assert_approx(c1: Complex, c2: Complex, tol: f64) {
        assert!(
            (c1.re - c2.re).abs() < tol,
            "Real parts differ: expected {}, got {} (diff {})",
            c2.re,
            c1.re,
            (c1.re - c2.re).abs()
        );
        assert!(
            (c1.im - c2.im).abs() < tol,
            "Imaginary parts differ: expected {}, got {} (diff {})",
            c2.im,
            c1.im,
            (c1.im - c2.im).abs()
        );
    }

    #[test]
    fn test_addition() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1 + c2;
        assert_eq!(result, Complex::new(4.0, 6.0));
    }

    #[test]
    fn test_subtraction() {
        let c1 = Complex::new(5.0, 7.0);
        let c2 = Complex::new(2.0, 3.0);
        let result = c1 - c2;
        assert_eq!(result, Complex::new(3.0, 4.0));
    }

    #[test]
    fn test_multiplication() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1 * c2;
        assert_eq!(result, Complex::new(-5.0, 10.0));
    }

    #[test]
    fn test_division() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1 / c2;
        assert!((result.re - 0.44).abs() < 1e-9);
        assert!((result.im - 0.08).abs() < 1e-9);
    }

    #[test]
    fn test_division_by_zero() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(0.0, 0.0);
        let result = c1 / c2;
        assert_eq!(result, Complex::new(0.0, 0.0));
    }

    #[test]
    fn test_pow() {
        let c = Complex::new(1.0, 1.0);
        let result = c.pow(3);
        assert_eq!(result, Complex::new(-2.0, 2.0));
    }

    #[test]
    fn test_abs_sq() {
        let c = Complex::new(3.0, -4.0);
        assert_eq!(c.abs_sq(), 25.0);
    }

    #[test]
    fn test_scalar_multiplication_complex_f64() {
        let c = Complex::new(1.5, -2.0);
        let scalar = 2.0;
        let result = c * scalar;
        assert_eq!(result, Complex::new(3.0, -4.0));
    }

    #[test]
    fn test_scalar_multiplication_f64_complex() {
        let c = Complex::new(1.5, -2.0);
        let scalar = 2.0;
        let result = scalar * c;
        assert_eq!(result, Complex::new(3.0, -4.0));
    }

    #[test]
    fn test_scalar_division_complex_f64() {
        let c = Complex::new(3.0, -4.0);
        let scalar = 2.0;
        let result = c / scalar;
        assert_eq!(result, Complex::new(1.5, -2.0));
    }

    #[test]
    fn test_scalar_division_complex_f64_by_zero() {
        let c = Complex::new(3.0, -4.0);
        let result = c / 0.0;
        assert_eq!(result, Complex::new(0.0, 0.0));
    }

    #[test]
    fn test_scalar_division_f64_complex() {
        let scalar = 5.0;
        let c = Complex::new(3.0, 4.0);
        let result = scalar / c;
        assert_eq!(result, Complex::new(0.6, -0.8));
    }

    #[test]
    fn test_scalar_division_f64_complex_by_zero() {
        let scalar = 5.0;
        let c = Complex::new(0.0, 0.0);
        let result = scalar / c;
        assert_eq!(result, Complex::new(0.0, 0.0));
    }

    #[test]
    fn test_newton_step_term() {
        let z = Complex::new(2.0, 0.0);
        let term = z.newton_step_term(3);
        assert!((term.re - 0.58333333333).abs() < 1e-9);
        assert_eq!(term.im, 0.0);
    }

    #[test]
    fn test_euler_identity() {
        let pi = std::f64::consts::PI;
        let exponent = Complex::i() * pi;
        let result = exponent.exp();
        assert_approx(result, -Complex::one(), 1e-9);
    }

    #[test]
    fn test_sqrt() {
        let c1 = Complex::new(-4.0, 0.0);
        assert_approx(c1.sqrt(), Complex::new(0.0, 2.0), 1e-9);

        let c2 = Complex::new(3.0, 4.0);
        assert_approx(c2.sqrt(), Complex::new(2.0, 1.0), 1e-9);
    }

    #[test]
    fn test_trig_identity() {
        let z = Complex::new(1.0, 2.0);
        let lhs = z.sin().square() + z.cos().square();
        assert_approx(lhs, Complex::one(), 1e-9);
    }

    #[test]
    fn test_hyperbolic_identity() {
        let z = Complex::new(1.0, 2.0);
        let lhs = z.cosh().square() - z.sinh().square();
        assert_approx(lhs, Complex::one(), 1e-9);
    }

    #[test]
    fn test_inverse_trig() {
        let z = Complex::new(0.5, 0.5);
        assert_approx(z.sin().asin(), z, 1e-9);
        assert_approx(z.cos().acos(), z, 1e-9);
        assert_approx(z.tan().atan(), z, 1e-9);
    }

    #[test]
    fn test_logarithm() {
        let z = Complex::new(10.0, 0.0);
        assert_approx(z.log(10.0), Complex::one(), 1e-9);
    }

    #[test]
    fn test_assignment_ops() {
        let mut c = Complex::new(1.0, 2.0);
        c += Complex::new(2.0, 3.0);
        assert_eq!(c, Complex::new(3.0, 5.0));

        c -= Complex::new(1.0, 1.0);
        assert_eq!(c, Complex::new(2.0, 4.0));

        c *= 2.0;
        assert_eq!(c, Complex::new(4.0, 8.0));

        c /= 4.0;
        assert_eq!(c, Complex::new(1.0, 2.0));
    }
}
