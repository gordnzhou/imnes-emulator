use ringbuf::RingBuffer;

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};

pub struct AudioPlayer {
    _stream: Stream,
    ring_buffer: ringbuf::Producer<f32>,

    pub master_volume: f32,

    pub sample_rate: u32,
    pub buffer_size: Option<u32>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let default_config = device.default_output_config().unwrap();

        let buffer_size = match default_config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max: _ } => cpal::BufferSize::Fixed(*min),
            _ => cpal::BufferSize::Default,
        };

        let length = match buffer_size {
            cpal::BufferSize::Fixed(size) => Some(size),
            _ => None,
        };

        let sample_rate = default_config.sample_rate();
        
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate,
            buffer_size,
        };

        let ring_buffer = RingBuffer::new(1024);
        let (producer, mut consumer) = ring_buffer.split();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            &config.into(), 
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    *sample = consumer.pop().unwrap_or(0.0);
                }
            },
            err_fn, 
            None
        ).unwrap();

        stream.play().expect("Failed to start stream");
        
        Self {
            _stream: stream,
            ring_buffer: producer,

            master_volume: 0.5,
            sample_rate: sample_rate.0,
            buffer_size: length,
        }
    }

    pub fn send_sample(&mut self, sample: f32) {
        let _ = self.ring_buffer.push(sample);
    }
}