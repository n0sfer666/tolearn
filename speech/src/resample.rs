use crate::pcm::{Pcm, RATE};

pub fn mixed(samples: &[f32], channels: u16) -> Vec<f32> {
    let channels = usize::from(channels.max(1));
    if channels == 1 {
        return samples.to_vec();
    }
    samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

pub fn to_16k(samples: Vec<f32>, rate: u32) -> Pcm {
    if rate == RATE || samples.is_empty() {
        return Pcm::new(samples);
    }
    let ratio = f64::from(rate) / f64::from(RATE);
    let wanted = ((samples.len() as f64) / ratio).floor() as usize;
    let last = samples.len() - 1;
    Pcm::new(
        (0..wanted)
            .map(|at| {
                let source = at as f64 * ratio;
                let left = source.floor() as usize;
                let right = (left + 1).min(last);
                let step = (source - left as f64) as f32;
                samples[left] * (1.0 - step) + samples[right] * step
            })
            .collect(),
    )
}
