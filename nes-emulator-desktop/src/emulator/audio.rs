use std::sync::mpsc::SyncSender;

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SampleRate, Stream, StreamConfig};

const AUDIO_SAMPLES: usize = 512;

pub struct AudioPlayer {
    _stream: Stream,
    audio_tx: SyncSender<[f32; AUDIO_SAMPLES]>,
    audio_buffer: [f32; AUDIO_SAMPLES],
    buffer_index: usize,
}

impl AudioPlayer {
    pub fn new(sampling_rate: u32) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(sampling_rate),
            buffer_size: cpal::BufferSize::Fixed(AUDIO_SAMPLES as u32),
        };

        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel::<[f32; AUDIO_SAMPLES]>(4);
        
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                match audio_rx.try_recv() {
                    Ok(buffer) => {
                        let len = data.len().min(buffer.len());
                        data[0..len].copy_from_slice(&buffer[0..len]);
                    }
                    Err(_) => {
                        for i in 0..data.len() {
                            data[i] = 0.0;
                        }
                    }
                } 
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None
        ).expect("Failed to build output stream");

        stream.play().expect("Failed to start stream");
        
        Self {
            _stream: stream,
            audio_tx,
            audio_buffer: [0.0; AUDIO_SAMPLES],
            buffer_index: 0,
        }
    }

    pub fn send_sample(&mut self, sample: f32) {
        self.audio_buffer[self.buffer_index] = sample;
        self.buffer_index += 1;

        if self.buffer_index == AUDIO_SAMPLES {
            let _ = self.audio_tx.try_send(self.audio_buffer);
            self.buffer_index = 0;
        }
    }
}