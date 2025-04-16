use std::sync::Arc;
use itertools::Itertools;

use rustfft::{
    num_complex::{Complex, Complex32}, Fft, FftPlanner
};

pub struct ProcessorOutput {
    pub left_fft: Vec<f32>,
    pub right_fft: Vec<f32>,
}

pub struct Processor {
    pub audio_buffer: (Vec<i16>, Vec<i16>),
    fft_window: usize,
    fft_processor: FftProcessor,
    pub fft_output_bins: usize,
    fft_io_vec: Vec<Complex32>,
    window: HannWindow,
}

pub enum Channel {LEFT, RIGHT}

impl Processor {
    pub fn new(sample_window: usize, fft_output_bins: usize) -> Self{
        Self {
            audio_buffer: (vec![0; sample_window], vec![0; sample_window]),
            fft_window: sample_window,
            fft_processor: FftProcessor::new(sample_window),
            window: HannWindow::new(sample_window),
            fft_output_bins,
            fft_io_vec: vec![Complex32::ZERO; sample_window * 2],
        }
    }

    pub fn process_samples(&mut self) -> ProcessorOutput {
        let left_fft = self.process_fft_samples(Channel::LEFT);
        let right_fft = self.process_fft_samples(Channel::RIGHT);

        ProcessorOutput { left_fft, right_fft }
    }

    fn process_fft_samples(&mut self, source: Channel) -> Vec<f32> {
        self.fft_io_vec.clear();

        let channel = match source {
            Channel::LEFT => &self.audio_buffer.0,
            Channel::RIGHT => &self.audio_buffer.1,
        };

        let samples_complex = channel.iter()
            .enumerate() 
            .map( |(u, d)| self.window.process((u, *d)))// Apply windowing
            .map( |x| Complex::new(x, 0.)) // Convert to Complex
            .chain(std::iter::repeat_n(Complex::ZERO, self.fft_window)) // Pad zeros
            .collect_into(&mut self.fft_io_vec); // Collect into vec 

        self.fft_processor.process_batch(samples_complex); 

        let parsed_samples = samples_complex
            .iter()
            .map(|x| x.norm() )// Take the norm
            .take(self.fft_window)  // Drop half the values
            .chunks(self.fft_window / self.fft_output_bins)// Average the value into bins
            .into_iter()
            .map(|chunk| 
                chunk.mean::<f32>()) // Take the mean
            .take(self.fft_output_bins)
            .collect();

        parsed_samples
    }
}

struct HannWindow {
    scales: Vec<f32>
}

impl HannWindow {
    fn new(window_size: usize) -> Self {
        use std::f32::consts::PI;

        let scales = (0..window_size)
            .map(|i| 
            (PI * i as f32 / window_size as f32).sin().powi(2)
            )
            .collect();

        Self { scales }
    }

    fn process(&self, sample: (usize, i16)) -> f32 {
        let (i, sample) = sample;
        return self.scales.get(i).unwrap() * sample as f32;
    }
}

struct FftProcessor {
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
}

impl FftProcessor {
    fn new(sample_window: usize) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(sample_window * 2);

        Self {
            fft,
            scratch: vec![Complex32::ZERO; sample_window * 2]
        }
    }

    fn process_batch(&mut self, input: &mut [Complex32]) {
        self.fft.process_with_scratch(input, &mut self.scratch);
    }
}

// struct PhaseSpaceProcessor {
//     history: HeapRb<f32>


// }

// Code from https://stackoverflow.com/questions/43921436/extend-iterator-with-a-mean-method
trait MeanExt: Iterator {
    fn mean<M>(self) -> M
    where
        M: Mean<Self::Item>,
        Self: Sized,
    {
        M::mean(self)
    }
}

impl<I: Iterator> MeanExt for I {}

trait Mean<A = Self> {
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
