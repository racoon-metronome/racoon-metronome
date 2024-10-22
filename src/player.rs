use crate::measure::{Beat, Measure, Sound, SoundType};
use crate::rhythm::Rhythm;
use rodio::source::Zero;
use rodio::{Decoder, Source};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct RodioPlayer {
    //TODO we probably don't need this to be arc/atomic anymore?
    bpm: Arc<AtomicU64>,
    // beats_per_measure: usize,
    measure: Measure,
    // stream: rodio::OutputStream,
    // stream_handle: rodio::OutputStreamHandle,
    pub sink: rodio::Sink,
    up: Vec<u8>,
    down: Vec<u8>,
    playing: bool,
}

impl Debug for RodioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RodioPlayer")
            .field("bpm", &self.bpm)
            .field("measure", &self.measure)
            .field("playing", &self.playing)
            .finish()
    }
}

impl RodioPlayer {
    pub fn new(
        up: Vec<u8>,
        down: Vec<u8>,
    ) -> (Self, rodio::OutputStream, rodio::OutputStreamHandle) {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let bpm = Arc::new(AtomicU64::new(120));
        // let bpm2 = bpm.clone();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        (
            Self {
                bpm,
                measure: Measure::default(),
                sink,
                up,
                down,
                playing: false,
            },
            stream,
            stream_handle,
        )
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.sink.play()
    }

    pub fn pause(&mut self) {
        self.playing = false;
        self.sink.pause()
    }

    pub fn push(&mut self) {
        if !self.playing {
            return;
        }
        let up = std::io::Cursor::new(self.up.clone());
        let up_source = Decoder::new(up)
            .unwrap()
            // .take_duration(Duration::from_millis(Self::METRONOME_TIME))
            .buffered();
        let up_length = up_source.total_duration().unwrap().as_millis();

        let down = std::io::Cursor::new(self.down.clone());
        let down_source = Decoder::new(down)
            .unwrap()
            // .take_duration(Duration::from_millis(Self::METRONOME_TIME))
            .buffered();
        let down_length = down_source.total_duration().unwrap().as_millis();

        self.measure.data.iter().enumerate().for_each(|(n, i)| {
            let s = match i.0[0].sound_type {
                SoundType::Up => up_source.clone(),
                SoundType::Mid => up_source.clone(), //TODO change
                SoundType::Down => down_source.clone(),
            };
            //     //TODO extract silence?
            let silence: Zero<i16> = Zero::new(s.channels(), s.sample_rate());
            let r = s.mix(silence).amplify(i.0[0].volume_modifier);
            self.sink.append(r.take_duration(Duration::from_nanos(
                i.0[0].duration.make_duration(self.bpm()),
            )));
        });
    }

    pub fn start(&mut self) {
        self.playing = true;
        self.push();
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.sink.stop();
    }

    pub fn set_bpm(&mut self, bpm: u64) {
        self.bpm.store(bpm, Ordering::Relaxed);
    }

    pub fn bpm(&self) -> u64 {
        self.bpm.load(Ordering::Relaxed)
    }

    pub fn playing(&self) -> bool {
        self.playing
    }
}
