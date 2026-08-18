use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};

use crate::error::SpeechError;
use crate::pcm::Pcm;
use crate::resample;

const TICK: Duration = Duration::from_millis(20);
const LONGEST: usize = 300;

#[derive(Debug)]
pub struct Microphone {
    stop: Arc<AtomicBool>,
    done: Receiver<Pcm>,
}

struct Tape {
    stream: Stream,
    heard: Arc<Mutex<Vec<f32>>>,
    rate: u32,
    channels: u16,
}

impl Microphone {
    pub fn open() -> Result<Self, SpeechError> {
        let (ready, told) = channel();
        let (finished, done) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);

        std::thread::spawn(move || record(&flag, &ready, &finished));

        match told.recv() {
            Ok(Ok(())) => Ok(Self { stop, done }),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(SpeechError::Deaf("поток записи не поднялся".to_owned())),
        }
    }

    pub fn stop(self) -> Result<Pcm, SpeechError> {
        self.stop.store(true, Ordering::Relaxed);
        let heard = self
            .done
            .recv()
            .map_err(|_| SpeechError::Deaf("поток записи оборвался".to_owned()))?;
        if heard.is_empty() {
            return Err(SpeechError::Silent);
        }
        Ok(heard)
    }
}

fn record(stop: &AtomicBool, ready: &Sender<Result<(), SpeechError>>, finished: &Sender<Pcm>) {
    let tape = match opened() {
        Ok(tape) => tape,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(())).is_err() {
        return;
    }
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(TICK);
    }
    let _ = finished.send(tape.finish());
}

fn opened() -> Result<Tape, SpeechError> {
    let device = cpal::default_host()
        .default_input_device()
        .ok_or_else(|| SpeechError::Deaf("в системе нет устройства записи".to_owned()))?;
    let chosen = device
        .default_input_config()
        .map_err(|error| SpeechError::Deaf(error.to_string()))?;
    let rate = chosen.sample_rate();
    let channels = chosen.channels();
    let room = LONGEST * rate as usize * usize::from(channels);
    let heard = Arc::new(Mutex::new(Vec::new()));
    let config = chosen.config();

    let taped = Arc::clone(&heard);
    let deaf = |_| ();
    let stream = match chosen.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            config,
            move |block: &[f32], _: &_| keep(&taped, block.iter().copied(), room),
            deaf,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            config,
            move |block: &[i16], _: &_| {
                keep(
                    &taped,
                    block.iter().map(|value| f32::from(*value) / 32_768.0),
                    room,
                );
            },
            deaf,
            None,
        ),
        other => {
            return Err(SpeechError::Unsupported(format!(
                "устройство отдаёт отсчёты в формате {other}"
            )));
        }
    };
    let stream = stream.map_err(|error| SpeechError::Deaf(error.to_string()))?;
    stream
        .play()
        .map_err(|error| SpeechError::Deaf(error.to_string()))?;

    Ok(Tape {
        stream,
        heard,
        rate,
        channels,
    })
}

fn keep(tape: &Mutex<Vec<f32>>, block: impl Iterator<Item = f32>, room: usize) {
    if let Ok(mut kept) = tape.lock() {
        for sample in block {
            if kept.len() >= room {
                return;
            }
            kept.push(sample);
        }
    }
}

impl Tape {
    fn finish(self) -> Pcm {
        drop(self.stream);
        let raw = self
            .heard
            .lock()
            .map(|kept| kept.clone())
            .unwrap_or_default();
        resample::to_16k(resample::mixed(&raw, self.channels), self.rate)
    }
}
