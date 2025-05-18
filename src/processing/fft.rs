use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};



pub struct HannWindow {
    scales: Vec<f32>
}

impl HannWindow {
    pub fn new(window_size: usize) -> Self {
        use std::f32::consts::PI;

        let scales = (0..window_size)
            .map(|i| 
            (PI * i as f32 / window_size as f32).sin().powi(2)
            )
            .collect();

        Self { scales }
    }

    pub fn process(&self, sample: (usize, i16)) -> f32 {
        let (i, sample) = sample;
        return self.scales.get(i).unwrap() * sample as f32;
    }
}

pub struct BMH4Window {
    scales: Vec<f32>
}

impl BMH4Window {
    pub fn new(window_size: usize) -> Self {
        use std::f32::consts::PI;

        let n_ = window_size as f32;

        let scales = (0..window_size)
            .map(|i| {
                    let n = i as f32;

                    0.35875 
                    - 0.48829 * (2. * PI / n_ * n) 
                    + 0.14128 * (2. * PI / n_ * 2. * n)
                    - 0.01168 * (2. * PI / n_ * 3. * n)
                })
            .collect();

        Self { scales }
    }

    pub fn process(&self, sample: (usize, i16)) -> f32 {
        self.process_f32((sample.0, sample.1 as f32))
    }

    pub fn process_f32(&self, sample: (usize, f32)) -> f32 {
        let (i, sample) = sample;
        return *self.scales.get(i).unwrap() as f32 * sample;
    }
}

pub struct MelFilter {
    filter: Vec<f32>
}

impl MelFilter {
    pub fn new(window_size: usize, nyquist: f32, sample_size:usize) -> Self {
        let start_m: Mel = 0f32.into();
        let end_m: Mel = nyquist.into();

        let mut endpoints = (0..=window_size)
        .map(|i| {
                (end_m - start_m) * i as f32
            })
        .map(|m| f32::from(m));

        let mut prev = endpoints.next().unwrap();
        let mut next = endpoints.next().unwrap();
        let mut mid = (next - prev) / 2.;
        
        let filter = (0..sample_size).into_iter()
            .map(|x| {
                let freq = nyquist / sample_size as f32 / 2. * x as f32;
                
                while freq > next {
                    prev = next;
                    next = endpoints.next().unwrap();
                    mid = (next - prev) / 2.;
                }

                if freq < mid {
                    freq - prev
                } else {
                    next - freq
                }
            })
            .collect();

        Self {
            filter,
        }
    }
    pub fn apply_filter(&self, samples: Vec<f32>) {
        
    }
}

#[derive(Clone, Copy)]
pub struct Mel(f32);

impl From<f32> for Mel {
    fn from(value: f32) -> Self {
        // From http://practicalcryptography.com/miscellaneous/machine-learning/guide-mel-frequency-cepstral-coefficients-mfccs/#eqn1
        Mel(1125. * (1. + value / 700.))
    }
}

impl From<Mel> for f32 {
    fn from(value: Mel) -> Self {
        700. * ((value.0/1125.).exp() - 1.)
    }
}

impl std::ops::Sub for Mel {
    type Output = Mel;

    fn sub(self, rhs: Self) -> Self::Output {
        Mel(self.0 - rhs.0)
    }
}

impl std::ops::Mul<f32> for Mel {
    type Output = Mel;
    
    fn mul(self, rhs: f32) -> Self::Output {
        return Mel(self.0 * rhs)
    }
}

pub struct FftProcessor {
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
}

impl FftProcessor {
    pub fn new(sample_window: usize, inverse: bool) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = if inverse {
            planner.plan_fft_inverse(sample_window * 2)
        } else {
            planner.plan_fft_forward(sample_window * 2)
        };

        Self {
            fft,
            scratch: vec![Complex32::ZERO; sample_window * 2]
        }
    }

    pub fn process_batch(&mut self, input: &mut [Complex32]) {
        self.fft.process_with_scratch(input, &mut self.scratch);
    }
}

// Code from https://stackoverflow.com/questions/43921436/extend-iterator-with-a-mean-method
pub trait MeanExt: Iterator {
    fn mean<M>(self) -> M
    where
        M: Mean<Self::Item>,
        Self: Sized,
    {
        M::mean(self)
    }
}

impl<I: Iterator> MeanExt for I {}

pub trait Mean<A = Self> {
    fn mean<I>(iter: I) -> Self
    where
        I: Iterator<Item = A>;
}

impl Mean for f32 {
    fn mean<I>(iter: I) -> Self
    where
        I: Iterator<Item = f32>,
    {
        let mut sum = 0.0;
        let mut count: usize = 0;

        for v in iter {
            sum += v;
            count += 1;
        }

        if count > 0 {
            sum / (count as f32)
        } else {
            0.0
        }
    }
}

impl<'a> Mean<&'a f32> for f32 {
    fn mean<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a f32>,
    {
        iter.copied().mean()
    }
}
