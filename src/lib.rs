#![feature(async_closure)]
use discovery_server::DiscoveryServer;
use local_ip_address::{list_afinet_netifas, local_ip};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use port_check::free_local_port_in_range;
use qrcode::QrCode;
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
    thread,
};
use tokio::runtime::Runtime;

use player::RodioPlayer;
use poem::{listener::TcpListener, middleware::AddData, web::Data, EndpointExt, Route, Server};
use poem_openapi::{param::Path, OpenApi, OpenApiService};
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

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Racoon {
    params: Arc<RacoonParams>,
}

#[derive(Params)]
struct RacoonParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,
}

impl Default for Racoon {
    fn default() -> Self {
        Self {
            params: Arc::new(RacoonParams::default()),
        }
    }
}

impl Default for RacoonParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(300, 180),
        }
    }
}

impl Plugin for Racoon {
    const NAME: &'static str = "Racoon Metronome";
    const VENDOR: &'static str = "Asayake";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "contact@asayake.xyz";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    //FIXME use this instead ? https://nih-plug.robbertvanderhelm.nl/nih_plug/context/process/trait.ProcessContext.html#tymethod.execute_background
    //FIXME save thread state and stop server in reset? more testing needed
    // It seems that deleting the vst doesn't affect the server
    // Also, the VST outputs aren't being used
    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        nih_dbg!(_audio_io_layout);
        thread::spawn(move || {
            // Resize buffers and perform other potentially expensive initialization operations here.
            // The `reset()` function is always called right after this function. You can remove this
            // function if you do not need it.

            // let up = fs::read("assets/up.wav").unwrap();
            // let down = fs::read("assets/down.wav").unwrap();
            let up = include_bytes!("../assets/up.wav").to_vec();
            let down = include_bytes!("../assets/down.wav").to_vec();
            let (player, _stream, _stream_handle) = RodioPlayer::new(up, down);

            let player = Arc::new(Mutex::new(player));

            let pusher = Mutex::new(Pusher::new(player.clone()));

            let state = Arc::new(State { player, pusher });

            let Some(port) = free_local_port_in_range(20000..=60000) else {
                panic!(
            "Couldn't find an open port, you shouldn't realistically be seeing this, exiting..."
        );
            };

            let api_service = OpenApiService::new(Api, "Racoon Metronome", "0.1")
                .server(&format!("http://localhost:{port}/api"));

            let ui = api_service.swagger_ui();
            let app = Route::new()
                .nest("/api", api_service)
                .nest("/doc", ui)
                .with(AddData::new(state));

            let _discovery_server = DiscoveryServer::new(port.into());
            let network_interfaces = list_afinet_netifas().unwrap();
            let mut loopback: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
            let likely_local = local_ip();
            // let mut locals = Vec::with_capacity(10);

            // let likely_local_qr = if let Ok(ip) = likely_local {
            //     let code = QrCode::new(&format!("{ip}:{port}")).unwrap();
            //     code.render::<unicode::Dense1x2>()
            //         .quiet_zone(true)
            //         .dark_color(unicode::Dense1x2::Light)
            //         .light_color(unicode::Dense1x2::Dark)
            //         .build()
            // } else {
            //     String::from("")
            // };

            // for (_, ip) in network_interfaces.iter() {
            //     if ip.is_loopback() {
            //         loopback = *ip;
            //     }
            //     if let IpAddr::V4(ipv4) = ip.to_canonical() {
            //         if ipv4.is_private() {
            //             locals.push(ip);
            //         }
            //     }
            // }

            // let mut qr_str = locals
            //     .iter()
            //     .map(|e| e.to_string())
            //     .collect::<Vec<String>>()
            //     .join(",");
            // qr_str.push_str(&format!(":{port}"));

            // let code = QrCode::new(&qr_str).unwrap();
            // let code = code
            //     .render::<unicode::Dense1x2>()
            //     .quiet_zone(true)
            //     .dark_color(unicode::Dense1x2::Light)
            //     .light_color(unicode::Dense1x2::Dark)
            //     .build();

            // println!("{loopback}",);
            // println!("{likely_local_qr}",);
            // println!("{code}",);

            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                Server::new(TcpListener::bind(&format!("0.0.0.0:{port}")))
                    .name("racoon")
                    .run(app)
                    .await
            });
        });

        true
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // NOTE: See `plugins/diopser/src/editor.rs` for an example using the generic UI widget

                    // This is a fancy widget that can get all the information it needs to properly
                    // display and modify the parameter from the parametr itself
                    // It's not yet fully implemented, as the text is missing.
                    ui.label("Some random integer");
                    // ui.add(widgets::ParamSlider::for_param(&params.some_int, setter));

                    // ui.label("Gain");
                    // ui.add(widgets::ParamSlider::for_param(&params.gain, setter));

                    ui.label(
                        "Also gain, but with a lame widget. Can't even render the value correctly!",
                    );
                    // This is a simple naieve version of a parameter slider that's not aware of how
                    // the parameters work
                    // ui.add(
                    //     egui::widgets::Slider::from_get_set(-30.0..=30.0, |new_value| {
                    //         match new_value {
                    //             Some(new_value_db) => {
                    //                 let new_value = util::gain_to_db(new_value_db as f32);

                    //                 setter.begin_set_parameter(&params.gain);
                    //                 setter.set_parameter(&params.gain, new_value);
                    //                 setter.end_set_parameter(&params.gain);

                    //                 new_value_db
                    //             }
                    //             None => util::gain_to_db(params.gain.value()) as f64,
                    //         }
                    //     })
                    //     .suffix(" dB"),
                    // );
                });
            },
        )
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Racoon {
    const CLAP_ID: &'static str = "com.asayake.racoon";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A Remote Controlled Metronome");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Racoon {
    const VST3_CLASS_ID: [u8; 16] = *b"RacoonMetronomee";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Racoon);
nih_export_vst3!(Racoon);
