use approx::relative_eq;
use rmpfit::{MPFitter, MPResult};

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    x: f64,
    n: f64,
}

impl Measurement {
    pub fn concurrency_and_latency(n: f64, r: f64) -> Measurement {
        Measurement { x: n / r, n }
    }

    pub fn concurrency_and_throughput(n: f64, x: f64) -> Measurement {
        Measurement { x, n }
    }

    pub fn throughput_and_latency(x: f64, r: f64) -> Measurement {
        Measurement { x, n: x * r }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Model {
    pub sigma: f64,
    pub kappa: f64,
    pub lambda: f64,
}

impl Model {
    pub fn build(measurements: &[Measurement]) -> Model {
        let fitter = ModelFitter(measurements.to_vec());
        let mut params = vec![
            0.1,
            0.01,
            measurements.iter().map(|m| m.x / m.n).fold(f64::NEG_INFINITY, f64::max),
        ];
        let res = fitter.mpfit(&mut params, None, &Default::default());
        assert!(res.is_ok());
        Model { sigma: params[0], kappa: params[1], lambda: params[2] }
    }

    pub fn throughput_at_concurrency(&self, n: f64) -> f64 {
        (self.lambda * n) / (1.0 + (self.sigma * (n - 1.0)) + (self.kappa * n * (n - 1.0)))
    }

    pub fn latency_at_concurrency(&self, n: f64) -> f64 {
        (1.0 + (self.sigma * (n - 1.0)) + (self.kappa * n * (n - 1.0))) / self.lambda
    }

    pub fn max_concurrency(&self) -> f64 {
        (((1.0 - self.sigma) / self.kappa).sqrt()).floor()
    }

    pub fn max_throughput(&self) -> f64 {
        self.throughput_at_concurrency(self.max_concurrency())
    }

    pub fn latency_at_throughput(&self, x: f64) -> f64 {
        (self.sigma - 1.0) / (self.sigma * x - self.lambda)
    }

    pub fn throughput_at_latency(&self, r: f64) -> f64 {
        ((self.sigma.powi(2)
            + self.kappa.powi(2)
            + 2.0 * self.kappa * (2.0 * self.lambda * r + self.sigma - 2.0))
            .sqrt()
            - self.kappa
            + self.sigma)
            / (2.0 * self.kappa * r)
    }

    pub fn concurrency_at_latency(&self, r: f64) -> f64 {
        (self.kappa - self.sigma
            + (self.sigma.powi(2)
                + self.kappa.powi(2)
                + 2.0 * self.kappa * ((2.0 * self.lambda * r) + self.sigma - 2.0))
                .sqrt())
            / (2.0 * self.kappa)
    }

    pub fn concurrency_at_throughput(&self, x: f64) -> f64 {
        self.latency_at_throughput(x) * x
    }

    pub fn contention_constrained(&self) -> bool {
        self.sigma > self.kappa
    }

    pub fn coherency_constrained(&self) -> bool {
        self.sigma < self.kappa
    }

    pub fn limitless(&self) -> bool {
        relative_eq!(self.kappa, 0.0)
    }
}

struct ModelFitter(Vec<Measurement>);

impl MPFitter for ModelFitter {
    fn eval(&self, params: &[f64], deviates: &mut [f64]) -> MPResult<()> {
        let model = Model { sigma: params[0], kappa: params[1], lambda: params[2] };
        for (d, m) in deviates.iter_mut().zip(self.0.iter()) {
            *d = m.x - model.throughput_at_concurrency(m.n);
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
    fn build() {
        let measurements: Vec<Measurement> = MEASUREMENTS
            .iter()
            .map(|&(n, x)| Measurement::concurrency_and_throughput(n, x))
            .collect();

        let model = Model::build(&measurements);

        assert_relative_eq!(model.sigma, 0.02671591, max_relative = 0.00001);
        assert_relative_eq!(model.kappa, 7.690945e-4, max_relative = 0.00001);
        assert_relative_eq!(model.lambda, 995.6486, max_relative = 0.00001);
    }

    const MEASUREMENTS: [(f64, f64); 32] = [
        (1.0, 955.16),
        (2.0, 1878.91),
        (3.0, 2688.01),
        (4.0, 3548.68),
        (5.0, 4315.54),
        (6.0, 5130.43),
        (7.0, 5931.37),
        (8.0, 6531.08),
        (9.0, 7219.8),
        (10.0, 7867.61),
        (11.0, 8278.71),
        (12.0, 8646.7),
        (13.0, 9047.84),
        (14.0, 9426.55),
        (15.0, 9645.37),
        (16.0, 9897.24),
        (17.0, 10097.6),
        (18.0, 10240.5),
        (19.0, 10532.39),
        (20.0, 10798.52),
        (21.0, 11151.43),
        (22.0, 11518.63),
        (23.0, 11806.0),
        (24.0, 12089.37),
        (25.0, 12075.41),
        (26.0, 12177.29),
        (27.0, 12211.41),
        (28.0, 12158.93),
        (29.0, 12155.27),
        (30.0, 12118.04),
        (31.0, 12140.4),
        (32.0, 12074.39),
    ];
}
