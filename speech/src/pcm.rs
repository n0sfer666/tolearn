pub const RATE: u32 = 16_000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pcm {
    samples: Vec<f32>,
}

impl Pcm {
    pub fn new(samples: Vec<f32>) -> Self {
        Self { samples }
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn seconds(&self) -> f64 {
        self.samples.len() as f64 / f64::from(RATE)
    }
}

impl From<Vec<f32>> for Pcm {
    fn from(samples: Vec<f32>) -> Self {
        Self::new(samples)
    }
}
