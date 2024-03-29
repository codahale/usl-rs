//! Types and functions for building Universal Scalability Law models from sets of observed
//! measurements.
//!
//! ```
//! use usl::{Model, Measurement};
//! let measurements = vec![
//!     Measurement::concurrency_and_throughput(1, 955.16),
//!     Measurement::concurrency_and_throughput(5, 4315.54),
//!     Measurement::concurrency_and_throughput(10, 7867.61),
//!     Measurement::concurrency_and_throughput(15, 9645.37),
//!     Measurement::concurrency_and_throughput(20, 10798.52),
//!     Measurement::concurrency_and_throughput(25, 12075.41),
//!     Measurement::concurrency_and_throughput(30, 12118.04),
//! ];
//! let model = Model::build(&measurements);
//! println!("{}", model.throughput_at_concurrency(100));
//! ```
//!

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications,
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::cognitive_complexity,
    clippy::missing_const_for_fn,
    clippy::needless_borrow
)]

use std::iter::FromIterator;
use std::time::Duration;

use approx::relative_eq;
use rmpfit::{MPFitter, MPResult};

/// A simultaneous measurement of at least two of the parameters of Little's Law: concurrency,
/// throughput, and latency. The third parameter is inferred from the other two.
///
/// [Measurement] instances can be created from pairs of dimensional types: `u32` for the number of
/// concurrent events, `f64` for the average rate of events, and `Duration` for the average duration
/// of events:
///
/// ```
/// use usl::Measurement;
/// use std::time::Duration;
///
/// let m: Measurement = (30, 1000.0).into();
/// let m: Measurement = (1000.0, 30).into();
/// let m: Measurement = (30, Duration::from_millis(200)).into();
/// let m: Measurement = (Duration::from_millis(200), 30).into();
/// let m: Measurement = (1000.0, Duration::from_millis(200)).into();
/// let m: Measurement = (Duration::from_millis(200), 1000.0).into();
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    /// The average number of concurrent events.
    pub n: f64,
    /// The long-term arrival rate of events, in events/sec.
    pub x: f64,
    /// The average duration of events, in seconds.
    pub r: f64,
}

impl Measurement {
    /// Create a measurement of a system's latency at a given level of concurrency. The throughput
    /// of the system is derived via Little's Law.
    #[must_use]
    pub fn concurrency_and_latency(n: u32, r: Duration) -> Measurement {
        let n = n.into();
        let r = r.as_secs_f64();
        Measurement { n, x: n / r, r } // L, λ=L/W, W
    }

    /// Create a measurement of a system's throughput at a given level of concurrency. The latency
    /// of the system is derived via Little's Law.
    #[must_use]
    pub fn concurrency_and_throughput(n: u32, x: f64) -> Measurement {
        let n = n.into();
        Measurement { n, x, r: n / x } // L, λ, W=L/λ
    }

    /// Create a measurement of a system's latency at a given level of throughput. The concurrency
    /// of the system is derived via Little's Law.
    #[must_use]
    pub fn throughput_and_latency(x: f64, r: Duration) -> Measurement {
        let r = r.as_secs_f64();
        Measurement { n: x * r, x, r } // L=λW, W, λ
    }
}

macro_rules! from_tuple {
    ($a:ty, $b:ty, $f:expr) => {
        impl From<($a, $b)> for Measurement {
            fn from(v: ($a, $b)) -> Self {
                $f(v.0, v.1)
            }
        }

        impl From<($b, $a)> for Measurement {
            fn from(v: ($b, $a)) -> Self {
                $f(v.1, v.0)
            }
        }
    };
}

from_tuple!(u32, f64, Measurement::concurrency_and_throughput);
from_tuple!(u32, Duration, Measurement::concurrency_and_latency);
from_tuple!(f64, Duration, Measurement::throughput_and_latency);

/// A Universal Scalability Law model.
///
/// Can be built from an explicit slice of [Measurement] instances via [Model::build] or via
/// `collect` on an iterator of [Measurement] instances or measurement tuples:
///
/// ```
/// let m: usl::Model = vec![
///     (10, 30.0),
///     (20, 80.0),
///     (30, 100.0),
///     (40, 140.0),
///     (50, 160.0),
///     (60, 222.0),
/// ].iter().collect();
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Model {
    /// The model's coefficient of contention, σ.
    pub sigma: f64,
    /// The model's coefficient of crosstalk/coherency, κ.
    pub kappa: f64,
    /// The model's coefficient of performance, λ.
    pub lambda: f64,
}

/// The minimum number of measurements required to build a model.
pub const MIN_MEASUREMENTS: usize = 6;

impl Model {
    /// Build a model whose parameters are generated from the given measurements.
    ///
    /// Finds a set of coefficients for the equation `y = λx/(1+σ(x-1)+κx(x-1))` which best fit the
    /// observed values using unconstrained least-squares regression. The resulting values for λ, κ,
    /// and σ are the parameters of the returned model.
    #[must_use]
    pub fn build(measurements: &[Measurement]) -> Model {
        assert!(
            measurements.len() >= MIN_MEASUREMENTS,
            "must have at least {} measurements",
            MIN_MEASUREMENTS
        );
        let fitter = ModelFitter(measurements.to_vec());
        let mut params = fitter.init_params();
        if let Err(err) = fitter.mpfit(&mut params, None, &Default::default()) {
            panic!("lma error: {}", err)
        }
        Model { sigma: params[0], kappa: params[1], lambda: params[2] }
    }

    /// Calculate the expected throughput given a number of concurrent events, `X(N)`.
    ///
    /// See "Practical Scalability Analysis with the Universal Scalability Law, Equation 3".
    #[must_use]
    pub fn throughput_at_concurrency(&self, n: u32) -> f64 {
        let n: f64 = n.into();
        (self.lambda * n) / (1.0 + (self.sigma * (n - 1.0)) + (self.kappa * n * (n - 1.0)))
    }

    /// Calculate the expected mean latency given a number of concurrent events, `R(N)`.
    ///
    /// See "Practical Scalability Analysis with the Universal Scalability Law, Equation 6".
    #[must_use]
    pub fn latency_at_concurrency(&self, n: u32) -> f64 {
        let n: f64 = n.into();
        (1.0 + (self.sigma * (n - 1.0)) + (self.kappa * n * (n - 1.0))) / self.lambda
    }

    /// Calculate the maximum expected number of concurrent events the system can handle, `N{max}`.
    ///
    /// See "Practical Scalability Analysis with the Universal Scalability Law, Equation 4".
    #[must_use]
    pub fn max_concurrency(&self) -> u32 {
        (((1.0 - self.sigma) / self.kappa).sqrt()).floor() as u32
    }

    /// Calculate the maximum expected throughput the system can handle, `X{max}`.
    #[must_use]
    pub fn max_throughput(&self) -> f64 {
        self.throughput_at_concurrency(self.max_concurrency())
    }

    /// Calculate the expected mean latency given a throughput, `R(X)`.
    ///
    /// See "Practical Scalability Analysis with the Universal Scalability Law, Equation 8".
    #[must_use]
    pub fn latency_at_throughput(&self, x: f64) -> f64 {
        (self.sigma - 1.0) / (self.sigma * x - self.lambda)
    }

    /// Calculate the expected throughput given a mean latency, `X(R)`.
    ///
    /// See "Practical Scalability Analysis with the Universal Scalability Law, Equation 9".
    #[must_use]
    pub fn throughput_at_latency(&self, r: Duration) -> f64 {
        let r = r.as_secs_f64();
        ((self.sigma.powi(2)
            + self.kappa.powi(2)
            + 2.0 * self.kappa * (2.0 * self.lambda * r + self.sigma - 2.0))
            .sqrt()
            - self.kappa
            + self.sigma)
            / (2.0 * self.kappa * r)
    }

    /// Calculate the expected number of concurrent events at a particular mean latency, `N(R)`.
    ///
    /// See "Practical Scalability Analysis with the Universal Scalability Law, Equation 10".
    #[must_use]
    pub fn concurrency_at_latency(&self, r: Duration) -> f64 {
        let r = r.as_secs_f64();
        (self.kappa - self.sigma
            + (self.sigma.powi(2)
                + self.kappa.powi(2)
                + 2.0 * self.kappa * ((2.0 * self.lambda * r) + self.sigma - 2.0))
                .sqrt())
            / (2.0 * self.kappa)
    }

    /// Calculate the expected number of concurrent events at a particular throughput, `N(X)`.
    #[must_use]
    pub fn concurrency_at_throughput(&self, x: f64) -> f64 {
        self.latency_at_throughput(x) * x
    }

    /// Whether or not the system is constrained by contention effects.
    #[must_use]
    pub fn is_contention_constrained(&self) -> bool {
        self.sigma > self.kappa
    }

    /// Whether or not the system is constrained by coherency effects.
    #[must_use]
    pub fn is_coherency_constrained(&self) -> bool {
        self.sigma < self.kappa
    }

    /// Whether or not the system is linearly scalable.
    #[must_use]
    pub fn is_limitless(&self) -> bool {
        relative_eq!(self.kappa, 0.0)
    }
}

impl FromIterator<Measurement> for Model {
    fn from_iter<T: IntoIterator<Item = Measurement>>(iter: T) -> Self {
        let measurements: Vec<Measurement> = iter.into_iter().collect();
        Model::build(&measurements)
    }
}

macro_rules! from_iterator {
    ($a:ty, $b:ty) => {
        impl<'a> FromIterator<&'a ($a, $b)> for Model {
            fn from_iter<T: IntoIterator<Item = &'a ($a, $b)>>(iter: T) -> Self {
                let measurements: Vec<Measurement> = iter.into_iter().map(|&m| m.into()).collect();
                Model::build(&measurements)
            }
        }
    };
}

from_iterator!(u32, f64);
from_iterator!(f64, u32);
from_iterator!(Duration, u32);
from_iterator!(u32, Duration);
from_iterator!(f64, Duration);
from_iterator!(Duration, f64);

struct ModelFitter(Vec<Measurement>);

impl ModelFitter {
    fn init_params(&self) -> Vec<f64> {
        vec![0.1, 0.01, self.0.iter().map(|m| m.x / m.n).fold(f64::NEG_INFINITY, f64::max)]
    }
}

impl MPFitter for ModelFitter {
    fn eval(&self, params: &[f64], deviates: &mut [f64]) -> MPResult<()> {
        let model = Model { sigma: params[0], kappa: params[1], lambda: params[2] };
        for (d, m) in deviates.iter_mut().zip(self.0.iter()) {
            *d = m.x - model.throughput_at_concurrency(m.n as u32);
        }
        Ok(())
    }

    fn number_of_points(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn measurement() {
        let m = Measurement::concurrency_and_latency(3, Duration::from_millis(600));
        assert_relative_eq!(m.n, 3.0);
        assert_relative_eq!(m.r, 0.6);
        assert_relative_eq!(m.x, 5.0);

        let m = Measurement::concurrency_and_throughput(3, 5.0);
        assert_relative_eq!(m.n, 3.0);
        assert_relative_eq!(m.r, 0.6);
        assert_relative_eq!(m.x, 5.0);

        let m = Measurement::throughput_and_latency(5.0, Duration::from_millis(600));
        assert_relative_eq!(m.n, 3.0);
        assert_relative_eq!(m.r, 0.6);
        assert_relative_eq!(m.x, 5.0);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn build() {
        let model: Model = MEASUREMENTS.iter().collect();

        assert_relative_eq!(model.sigma, 0.02671591, max_relative = ACCURACY);
        assert_relative_eq!(model.kappa, 7.690945e-4, max_relative = ACCURACY);
        assert_relative_eq!(model.lambda, 995.6486, max_relative = ACCURACY);
        assert_eq!(model.max_concurrency(), 35);
        assert_relative_eq!(model.max_throughput(), 12341.7454, max_relative = ACCURACY);
        assert!(!model.is_coherency_constrained());
        assert!(model.is_contention_constrained());
        assert!(!model.is_limitless());

        assert_relative_eq!(model.latency_at_concurrency(1), 0.0010043702162450092);
        assert_relative_eq!(model.latency_at_concurrency(20), 0.0018077244442155811);
        assert_relative_eq!(model.latency_at_concurrency(35), 0.002835903510841524);

        assert_relative_eq!(model.throughput_at_concurrency(1), 995.648799442353);
        assert_relative_eq!(model.throughput_at_concurrency(20), 11063.633101824058);
        assert_relative_eq!(model.throughput_at_concurrency(35), 12341.74571391328);

        assert_relative_eq!(model.concurrency_at_throughput(955.0), 0.958099855673978);
        assert_relative_eq!(model.concurrency_at_throughput(11048.0), 15.35043561102983);
        assert_relative_eq!(model.concurrency_at_throughput(12201.0), 17.732208293896793);

        assert_relative_eq!(
            model.throughput_at_latency(Duration::from_millis(30)),
            7047.844027581335
        );
        assert_relative_eq!(
            model.throughput_at_latency(Duration::from_millis(40)),
            6056.536321602774
        );
        assert_relative_eq!(
            model.throughput_at_latency(Duration::from_millis(50)),
            5387.032125730636
        );

        assert_relative_eq!(model.latency_at_throughput(7000.0), 0.0012036103337889738);
        assert_relative_eq!(model.latency_at_throughput(6000.0), 0.001165116923601453);
        assert_relative_eq!(model.latency_at_throughput(5000.0), 0.0011290093731056857);

        assert_relative_eq!(
            model.concurrency_at_latency(Duration::from_millis(30)),
            177.69840792284043
        );
        assert_relative_eq!(
            model.concurrency_at_latency(Duration::from_millis(40)),
            208.52453995951137
        );
        assert_relative_eq!(
            model.concurrency_at_latency(Duration::from_millis(50)),
            235.61469338193223
        );
    }

    const ACCURACY: f64 = 0.00001;

    const MEASUREMENTS: [(u32, f64); 32] = [
        (1, 955.16),
        (2, 1878.91),
        (3, 2688.01),
        (4, 3548.68),
        (5, 4315.54),
        (6, 5130.43),
        (7, 5931.37),
        (8, 6531.08),
        (9, 7219.8),
        (10, 7867.61),
        (11, 8278.71),
        (12, 8646.7),
        (13, 9047.84),
        (14, 9426.55),
        (15, 9645.37),
        (16, 9897.24),
        (17, 10097.6),
        (18, 10240.5),
        (19, 10532.39),
        (20, 10798.52),
        (21, 11151.43),
        (22, 11518.63),
        (23, 11806.0),
        (24, 12089.37),
        (25, 12075.41),
        (26, 12177.29),
        (27, 12211.41),
        (28, 12158.93),
        (29, 12155.27),
        (30, 12118.04),
        (31, 12140.4),
        (32, 12074.39),
    ];
}
