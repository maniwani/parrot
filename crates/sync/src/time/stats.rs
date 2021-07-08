use float_ord::FloatOrd;

/// Finite-length series of data points stored in chronological order. Calculates their mean and variance.
pub struct TimeSeries {
    mean: f64,
    var_sum: f64,
    variance: f64,
    index: usize,
    samples: Vec<f64>,
}

impl TimeSeries {
    /// Constructs a new `TimeSeries` instance with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            mean: 0.0,
            var_sum: 0.0,
            variance: 0.0,
            index: 0,
            samples: Vec::with_capacity(capacity),
        }
    }

    /// Adds a new data point. If the series is at full capacity, the oldest data point is removed.
    pub fn push(&mut self, value: f64) {
        assert!(value.is_finite());
        let prev_mean = self.mean;
        if self.samples.len() < self.samples.capacity() {
            self.index += 1;
            self.samples.push(value);
            self.mean += (value - prev_mean) / (self.samples.len() as f64);
            self.var_sum += (value - self.mean) * (value - prev_mean);
        } else {
            self.index = (self.index + 1) % self.samples.len();
            let removed = self.samples[self.index];
            self.samples[self.index] = value;
            self.mean += (value - removed) / (self.samples.len() as f64);
            self.var_sum += (value - self.mean + removed - prev_mean) * (value - removed);
        }
        
        assert!(
            self.var_sum.is_sign_positive() && self.var_sum.is_finite(),
            "negative variance"
        );

        if self.samples.len() > 1 {
            // sample (unbiased) variance
            self.variance = self.var_sum / ((self.samples.len() - 1) as f64);
        }
    }

    /// Returns the value of the newest data point.
    pub fn latest(&self) -> f64 {
        self.samples[self.index]
    }

    /// Returns the smallest value among the currently stored data points.
    pub fn min(&self) -> Option<f64> {
        self.samples
            .iter()
            .cloned()
            .map(|f| FloatOrd(f))
            .min()
            .and_then(|ord| {
                let FloatOrd(f) = ord;
                Some(f)
            })
    }

    /// Returns the largest value among the currently stored data points.
    pub fn max(&self) -> Option<f64> {
        self.samples
            .iter()
            .cloned()
            .map(|f| FloatOrd(f))
            .max()
            .and_then(|ord| {
                let FloatOrd(f) = ord;
                Some(f)
            })
    }

    /// Returns the mean value of the currently stored data points.
    #[inline]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Returns the variance of the currently stored data points.
    #[inline]
    pub fn variance(&self) -> f64 {
        self.variance
    }

    /// Returns the standard deviation of the currently stored data points.
    #[inline]
    pub fn standard_deviation(&self) -> f64 {
        self.variance.sqrt()
    }

    pub fn cdf(&self, value: f64) -> f64 {
        todo!()
    }

    pub fn cdf_from_mean(&self, value: f64) -> f64 {
        todo!()
    }

    pub fn inverse_cdf(&self, p: f64) -> f64 {
        todo!()
    }

    pub fn inverse_cdf_from_mean(&self, p: f64) -> f64 {
        todo!()
    }
}
