use std::sync::{mpsc::SyncSender, Arc, Mutex};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, 
    SampleRate, 
    Stream, 
    StreamConfig
};

const DEFAULT_BUFFER_SIZE: usize = 512;

pub struct AudioPlayer {
    _stream: Stream,
    audio_tx: SyncSender<Vec<f32>>,
    audio_buffer: Vec<f32>,
    callback_data_len: Arc<Mutex<usize>>,
    buffer_index: usize,

    pub master_volume: f32,
    sample_rate: u32,
    buffer_size: usize,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let default_config = device.default_output_config().unwrap();

        let sample_rate = default_config.sample_rate().0;

        let callback_data_len = Arc::new(Mutex::new(DEFAULT_BUFFER_SIZE));
        let callback_data_len_clone = Arc::clone(&callback_data_len);

        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(4);
        
        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                match audio_rx.try_recv() {
                    Ok(buffer) => {
                        *callback_data_len_clone.lock().unwrap() = data.len();

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
        ).expect("Failed to build audio stream");

        stream.play().expect("Failed to start stream");
        
        Self {
            _stream: stream,
            audio_tx,
            audio_buffer: vec![0.0; DEFAULT_BUFFER_SIZE],
            buffer_index: 0,
            buffer_size: DEFAULT_BUFFER_SIZE,
            
            master_volume: 0.5,
            sample_rate,
            callback_data_len,
        }
    }

    pub fn send_sample(&mut self, sample: f32) {
        self.audio_buffer[self.buffer_index] = sample * self.master_volume;
        self.buffer_index += 1;

        if self.buffer_index == self.buffer_size {
            let _ = self.audio_tx.try_send(self.audio_buffer.clone());

            // Adjust buffer size to match the length of the previous callback's data
            self.buffer_index = 0;
            self.buffer_size = *self.callback_data_len.lock().unwrap();
            self.audio_buffer.resize(self.buffer_size, 0.0);
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }
}