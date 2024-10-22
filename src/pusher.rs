use std::{
    sync::{Arc, Mutex},
    thread::{self, park_timeout, JoinHandle},
    time::Duration,
};

use crate::player::RodioPlayer;

pub struct Pusher {
    player: Arc<Mutex<RodioPlayer>>,
    thread: JoinHandle<()>,
}

//TODO park forever when not playing, start etc unparks
impl Pusher {
    pub fn new(player: Arc<Mutex<RodioPlayer>>) -> Self {
        let player2 = player.clone();
        let thread = thread::Builder::new()
            .spawn(move || {
                let mut bpm;
                // let mut bpm = 120;
                let mut beats_per_measure = 4;
                loop {
                    let mut lock = player2.lock().unwrap();

                    lock.push();
                    bpm = lock.bpm();
                    let pos = lock.sink.get_pos();

                    drop(lock);

                    let sleep_duration = (60_000 / bpm) * (beats_per_measure - 1);
                    // println!("sleep {sleep_duration}",);
                    // println!("pos {pos:?}",);
                    //TODO spin sleep here?
                    park_timeout(Duration::from_millis(sleep_duration));
                }
            })
            .unwrap();

        Self { player, thread }
    }
    pub fn unpark(&self) {
        self.thread.thread().unpark();
    }
}
