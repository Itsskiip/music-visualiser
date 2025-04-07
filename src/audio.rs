use std::{fs::File, io::BufReader, time::Duration};

use itertools::Itertools;
use rodio::{Decoder, OutputStream, Sink, Source};
use ringbuf::{traits::{Consumer, RingBuffer}, HeapRb};

pub struct Audio<'a> {
    sink: Sink,
    _stream: OutputStream,

    last_sample: Duration,
    data: Box<dyn Iterator<Item=i16> + 'a>,
    buffer_size: usize,
    internal_buffer: HeapRb<(i16, i16)>,
    sample_rate: usize,
    channels: usize,
}

impl Audio<'_> {
    pub fn new(file_name: &str, buffer_size: usize) -> Self {
        let file = File::open(file_name).unwrap();

        let source_data = Decoder::new(BufReader::new(file)).unwrap();
        
        let channels = source_data.channels() as usize;
        
        let file2 = File::open(file_name).unwrap();        
        let source_audio = Decoder::new(BufReader::new(file2)).unwrap();
        let sample_rate = source_audio.sample_rate() as usize;
        
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        
        let sink = Sink::try_new(&stream_handle).unwrap();
        
        sink.pause();
        sink.set_volume(0.5);
        sink.append(source_audio);

        Self {
            sink,
            _stream,
            last_sample: Duration::ZERO,
            data: Box::new(source_data),
            buffer_size,
            internal_buffer: HeapRb::new(buffer_size),
            sample_rate,
            channels,
        }
    }

    pub fn play(&mut self) {
        self.sink.play();
    }

    pub fn get_samples(&mut self, slice: &mut [(i16, i16)]) {
        let pos = self.sink.get_pos();
        let dif = pos - self.last_sample;
        self.last_sample = pos;

        let frames = (dif.as_secs_f32() * self.sample_rate as f32).round() as i32;

        let skip_neg = frames - self.buffer_size as i32;
        
        let skip = if skip_neg > 0 {skip_neg as usize} else {0};
        let take = if skip_neg < 0 {frames as usize} else {self.buffer_size};

        let binding = self.data.by_ref()
            .skip(skip * self.channels)
            .take(take * self.channels)
            .chunks(self.channels);
        let data = binding
            .into_iter()
            .map(|mut chunk| {
                let left = chunk.next().unwrap();
                let right = chunk.next().unwrap_or(left);
                (left, right)
            });
        
        self.internal_buffer.push_iter_overwrite(data);
        
        self.internal_buffer.peek_slice(slice);
    }   
}