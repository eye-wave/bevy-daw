pub mod tone;

pub trait AudioNode: Send {
    fn process(&mut self, sample_pos: u32, output: &mut [f32]);
}
