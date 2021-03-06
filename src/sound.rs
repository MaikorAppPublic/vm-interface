use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use maikor_vm_core::AudioPlayer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

pub struct CpalPlayer {
    buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    sample_rate: u32,
}

impl CpalPlayer {
    pub fn get() -> Option<(CpalPlayer, Arc<AtomicBool>, JoinHandle<()>)> {
        let device = match cpal::default_host().default_output_device() {
            Some(e) => e,
            None => return None,
        };

        // We want a config with:
        // chanels = 2
        // SampleFormat F32
        // Rate at around 44100

        let wanted_samplerate = cpal::SampleRate(44100);
        let supported_configs = match device.supported_output_configs() {
            Ok(e) => e,
            Err(_) => return None,
        };
        let mut supported_config = None;
        for f in supported_configs {
            if f.channels() == 2 && f.sample_format() == cpal::SampleFormat::F32 {
                if f.min_sample_rate() <= wanted_samplerate
                    && wanted_samplerate <= f.max_sample_rate()
                {
                    supported_config = Some(f.with_sample_rate(wanted_samplerate));
                } else {
                    supported_config = Some(f.with_max_sample_rate());
                }
                break;
            }
        }
        #[allow(clippy::question_mark)] //very weird syntax
        if supported_config.is_none() {
            return None;
        }

        let selected_config = supported_config.unwrap();

        let sample_format = selected_config.sample_format();
        let config: cpal::StreamConfig = selected_config.into();

        let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);

        let shared_buffer = Arc::new(Mutex::new(Vec::new()));
        let stream_buffer = shared_buffer.clone();

        let player = CpalPlayer {
            buffer: shared_buffer,
            sample_rate: config.sample_rate.0,
        };

        let keep_alive = Arc::new(AtomicBool::new(true));
        let thread_keep_alive = keep_alive.clone();

        let handle = thread::spawn(move || {
            let stream = match sample_format {
                cpal::SampleFormat::F32 => device.build_output_stream(
                    &config,
                    move |data: &mut [f32], _callback_info: &cpal::OutputCallbackInfo| {
                        cpal_thread(data, &stream_buffer)
                    },
                    err_fn,
                ),
                cpal::SampleFormat::U16 => device.build_output_stream(
                    &config,
                    move |data: &mut [u16], _callback_info: &cpal::OutputCallbackInfo| {
                        cpal_thread(data, &stream_buffer)
                    },
                    err_fn,
                ),
                cpal::SampleFormat::I16 => device.build_output_stream(
                    &config,
                    move |data: &mut [i16], _callback_info: &cpal::OutputCallbackInfo| {
                        cpal_thread(data, &stream_buffer)
                    },
                    err_fn,
                ),
            }
            .unwrap();

            stream.play().unwrap();

            while thread_keep_alive.load(Ordering::SeqCst) {
                sleep(Duration::from_millis(1));
            }

            eprintln!("Audio stream thread died");
        });

        Some((player, keep_alive, handle))
    }
}

fn cpal_thread<T: cpal::Sample>(outbuffer: &mut [T], audio_buffer: &Arc<Mutex<Vec<(f32, f32)>>>) {
    let mut inbuffer = audio_buffer.lock().unwrap();
    let outlen = ::std::cmp::min(outbuffer.len() / 2, inbuffer.len());
    for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
        outbuffer[i * 2] = cpal::Sample::from(&in_l);
        outbuffer[i * 2 + 1] = cpal::Sample::from(&in_r);
    }
}

impl AudioPlayer for CpalPlayer {
    fn play(&mut self, buf_left: &[f32], buf_right: &[f32]) {
        debug_assert!(buf_left.len() == buf_right.len());

        let mut buffer = self.buffer.lock().unwrap();

        for (l, r) in buf_left.iter().zip(buf_right) {
            if buffer.len() > self.sample_rate as usize {
                // Do not fill the buffer with more than 1 second of data
                // This speeds up the resync after the turning on and off the speed limiter
                return;
            }
            buffer.push((*l, *r));
        }
    }

    fn samples_rate(&self) -> u32 {
        self.sample_rate
    }

    fn underflowed(&self) -> bool {
        (*self.buffer.lock().unwrap()).is_empty()
    }
}
