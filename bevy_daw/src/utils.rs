pub trait Note {
    fn to_freq(&self) -> f32;
}

pub struct MidiNote(u8);
impl MidiNote {
    pub fn new(note: u8) -> Self {
        if note > 127 {
            panic!()
        }

        Self(note)
    }
}

impl Note for MidiNote {
    fn to_freq(&self) -> f32 {
        440.0 * 2f32.powf((self.0 as f32 - 69.0) / 12.0)
    }
}

impl From<MidiNote> for f32 {
    fn from(value: MidiNote) -> Self {
        value.to_freq()
    }
}
