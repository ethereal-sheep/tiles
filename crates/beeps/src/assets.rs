use std::path::Path;
use std::sync::Arc;

use crate::source::sample_player::SampleData;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AssetId(pub(crate) u32);

pub struct AssetStore {
    samples: Vec<Arc<SampleData>>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn load_wav(&mut self, path: &Path) -> Result<AssetId, String> {
        let reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
        let spec = reader.spec();

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .map(|s| s.unwrap_or(0) as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => {
                reader
                    .into_samples::<f32>()
                    .map(|s| s.unwrap_or(0.0))
                    .collect()
            }
        };

        let data = Arc::new(SampleData {
            samples,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
        });

        let id = AssetId(self.samples.len() as u32);
        self.samples.push(data);
        Ok(id)
    }

    pub fn load_wav_from_memory(&mut self, data: &[u8]) -> Result<AssetId, String> {
        let cursor = std::io::Cursor::new(data);
        let reader = hound::WavReader::new(cursor).map_err(|e| e.to_string())?;
        let spec = reader.spec();

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .map(|s| s.unwrap_or(0) as f32 / max_val)
                    .collect()
            }
            hound::SampleFormat::Float => {
                reader
                    .into_samples::<f32>()
                    .map(|s| s.unwrap_or(0.0))
                    .collect()
            }
        };

        let data = Arc::new(SampleData {
            samples,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
        });

        let id = AssetId(self.samples.len() as u32);
        self.samples.push(data);
        Ok(id)
    }

    pub fn get(&self, id: AssetId) -> Option<Arc<SampleData>> {
        self.samples.get(id.0 as usize).cloned()
    }
}
