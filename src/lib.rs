use std::sync::{Arc, Mutex};

use player::RodioPlayer;
use poem::web::Data;
use poem_openapi::{param::Path, OpenApi};
use pusher::Pusher;

pub mod discovery_server;
pub mod measure;
pub mod player;
pub mod pusher;
pub mod rhythm;

pub struct Api;

pub struct State {
    pub player: Arc<Mutex<RodioPlayer>>,
    pub pusher: Mutex<Pusher>,
}

type AppState = Arc<State>;

#[OpenApi]
impl Api {
    #[oai(path = "/health", method = "get")]
    async fn health(&self) {
        #[cfg(debug_assertions)]
        println!("->> /health - ");
    }

    #[oai(path = "/start", method = "post")]
    async fn start(&self, state: Data<&AppState>) {
        #[cfg(debug_assertions)]
        println!("->> /start - ");

        let mut player = state.player.lock().unwrap();
        player.start();
    }

    #[oai(path = "/play", method = "post")]
    async fn play(&self, state: Data<&AppState>) {
        #[cfg(debug_assertions)]
        println!("->> /play - ");

        let mut player = state.player.lock().unwrap();
        player.play();
    }

    #[oai(path = "/pause", method = "post")]
    async fn pause(&self, state: Data<&AppState>) {
        #[cfg(debug_assertions)]
        println!("->> /pause - ");

        let mut player = state.player.lock().unwrap();
        player.pause();
    }

    #[oai(path = "/stop", method = "post")]
    async fn stop(&self, state: Data<&AppState>) {
        #[cfg(debug_assertions)]
        println!("->> /stop - ");

        let mut player = state.player.lock().unwrap();
        player.stop();
    }

    #[oai(path = "/push", method = "post")]
    async fn push(&self, state: Data<&AppState>) {
        #[cfg(debug_assertions)]
        println!("->> /push - ");

        let mut player = state.player.lock().unwrap();
        player.push();
    }

    #[oai(path = "/set_bpm/:bpm", method = "post")]
    async fn set_bpm(&self, bpm: Path<u64>, state: Data<&AppState>) {
        #[cfg(debug_assertions)]
        println!("->> /set_bpm - bpm:{} ", *bpm);

        let mut player = state.player.lock().unwrap();
        player.set_bpm(*bpm);
        if player.playing() {
            player.stop();
            player.start();
        }
        state.pusher.lock().unwrap().unpark();
        // player.play();
    }

    // #[oai(path = "/set_rhythm/:rhythm", method = "post")]
    // async fn set_rhythm(&self, rhythm: Path<String>, state: Data<&Player>) {
    //     Rhythm::from_str(&rhythm).and_then(|r| {
    //         let mut player = state.0.lock().unwrap();
    //         player.set_rhythm(r);
    //         Ok(())
    //     });
    // }
}
