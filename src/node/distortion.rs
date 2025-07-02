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

#[cfg(test)]
mod test {
    use super::{AudioNode, DistortionNode, DistortionType};
    use crate::engine::SAMPLE_RATE;
    use crate::node::nodes::ToneGeneratorNode;
    use crate::node::test_utils::test::*;

    #[test]
    fn plot_tone_generator() {
        let freq = SAMPLE_RATE as f32 / 2048.0;
        let mut tone = ToneGeneratorNode::new(freq * 2.0, 0.5);
        let mut dist = DistortionNode::new(10.0, 1.0, DistortionType::HardClip);

        let modes = [
            (DistortionType::HardClip, "hardclip"),
            (DistortionType::SineWarp, "sinewarp"),
            (DistortionType::SoftClip, "softclip"),
        ];

        for (mode, name) in modes {
            let mut buffer = [0.0; 2048];

            dist.mode = mode;

            tone.process(0, &mut buffer);
            dist.process(0, &mut buffer);

            node_test_suite(&buffer, 1024, &format!("dist-{name}"));
        }
    }
}
