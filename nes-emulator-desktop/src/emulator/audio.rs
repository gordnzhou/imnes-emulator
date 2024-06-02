use std::sync::mpsc::{Receiver, SyncSender};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, BuildStreamError, Device, SampleRate, Stream, StreamConfig};

const AUDIO_SAMPLES: usize = 512;

pub struct AudioPlayer {
    stream: Stream,
    device: Device,
    audio_tx: SyncSender<[f32; AUDIO_SAMPLES]>,
    audio_buffer: [f32; AUDIO_SAMPLES],
    buffer_index: usize,
    pub master_volume: f32,
    pub sample_rate: u32,
}

impl AudioPlayer {
    pub fn new(sample_rate: u32) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let mut sample_rate = sample_rate;

        if let Ok(ocs) = device.supported_output_configs() {

            let mut new_sample_rate = None;

            for output_config in ocs {
                let min_sr = output_config.min_sample_rate().0;
                let max_sr = output_config.max_sample_rate().0;
                if min_sr <= sample_rate && max_sr <= sample_rate {
                    break
                } else {
                    new_sample_rate = Some((min_sr + max_sr) / 2);
                }
            }

            if let Some(new_sample_rate) = new_sample_rate {
                println!("Unable to use default sample rate of {}, using {} instead", sample_rate, new_sample_rate);
                sample_rate = new_sample_rate;
            };
        }

        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel::<[f32; AUDIO_SAMPLES]>(4);
        let stream = Self::build_audio_stream(audio_rx, &device, sample_rate).expect("Failed to build initial audio stream");

        stream.play().expect("Failed to start stream");
        
        Self {
            stream,
            device,
            audio_tx,
            audio_buffer: [0.0; AUDIO_SAMPLES],
            buffer_index: 0,
            master_volume: 0.5,
            sample_rate
        }
    }

    pub fn adjust_sample_rate(&mut self, sample_rate: u32) -> Result<(), BuildStreamError> {
        self.stream.pause().expect("Failed to pause stream");
    
        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel::<[f32; AUDIO_SAMPLES]>(4);

        let stream = Self::build_audio_stream(audio_rx, &self.device, sample_rate)?;
    
        self.stream.play().expect("Failed to start stream");

        self.stream = stream;
        self.audio_tx = audio_tx;
        self.sample_rate = sample_rate;

        Ok(())
    }

    fn build_audio_stream(audio_rx: Receiver<[f32; AUDIO_SAMPLES]>, device: &Device, sample_rate: u32) -> Result<Stream, BuildStreamError> { 
        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(AUDIO_SAMPLES as u32),
        };

        device.build_output_stream(
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
        )
    }

    pub fn send_sample(&mut self, sample: f32) {
        self.audio_buffer[self.buffer_index] = sample * self.master_volume;
        self.buffer_index += 1;

        if self.buffer_index == AUDIO_SAMPLES {
            let _ = self.audio_tx.try_send(self.audio_buffer);
            self.buffer_index = 0;
        }
    }
}