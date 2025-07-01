use crate::node::AudioNode;
use std::f32::consts::PI;

const SOFT_CLIP_NORM: f32 = 2.0 / PI;

#[derive(Debug)]
pub enum DistortionType {
    SoftClip,
    HardClip,
    SineWarp,
}

#[derive(Debug)]
pub struct DistortionNode {
    gain: f32,
    ceil: f32,
    mode: DistortionType,
}

impl DistortionNode {
    pub fn new(gain: f32, ceil: f32, mode: DistortionType) -> Self {
        Self { gain, mode, ceil }
    }
}

impl AudioNode for DistortionNode {
    fn process(&mut self, _sample_pos: u32, output: &mut [f32]) {
        match self.mode {
            DistortionType::SoftClip => {
                for sample in output {
                    *sample = SOFT_CLIP_NORM * (self.gain * *sample).atan() * self.ceil;
                }
            }
            DistortionType::HardClip => {
                for sample in output {
                    *sample = (self.gain * *sample).clamp(-self.ceil, self.ceil);
                }
            }
            DistortionType::SineWarp => {
                for sample in output {
                    *sample = (self.gain * *sample).sin() * self.ceil;
                }
            }
        }
    }
}

impl Default for DistortionNode {
    fn default() -> Self {
        Self::new(1.0, 1.0, DistortionType::HardClip)
    }
}
