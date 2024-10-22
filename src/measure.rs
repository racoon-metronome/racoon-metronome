use crate::rhythm::Rhythm;

#[derive(Debug, Clone)]
pub struct Measure {
    pub beats_per_measure: usize,
    pub data: Vec<Beat>,
}

impl Default for Measure {
    fn default() -> Self {
        Measure {
            beats_per_measure: 4,
            data: vec![
                Beat(vec![Sound {
                    sound_type: SoundType::Up,
                    duration: Rhythm::Quarter,
                    volume_modifier: 3.0,
                    hidden: false,
                }]),
                Beat(vec![Sound {
                    sound_type: SoundType::Down,
                    duration: Rhythm::Quarter,
                    volume_modifier: 1.0,
                    hidden: false,
                }]),
                Beat(vec![Sound {
                    sound_type: SoundType::Down,
                    duration: Rhythm::Quarter,
                    volume_modifier: 1.0,
                    hidden: false,
                }]),
                Beat(vec![Sound {
                    sound_type: SoundType::Down,
                    duration: Rhythm::Quarter,
                    volume_modifier: 1.0,
                    hidden: false,
                }]),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Beat(pub Vec<Sound>);

#[derive(Debug, Clone)]
pub enum SoundType {
    Up,
    Mid,
    Down,
}

#[derive(Debug, Clone)]
pub struct Sound {
    pub sound_type: SoundType,
    pub duration: Rhythm,
    pub volume_modifier: f32,
    // pitch_modifier?
    pub hidden: bool,
    // rhythm? and compute duration from?
}
