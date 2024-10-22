use std::{
    fs::{self, File},
    io::Write,
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
};

use local_ip_address::{list_afinet_netifas, local_ip};
use poem::{listener::TcpListener, middleware::AddData, web::Data, EndpointExt, Route, Server};
use poem_openapi::{param::Path, OpenApi, OpenApiService};
use port_check::free_local_port_in_range;
use qrcode::{render::unicode, QrCode};
use racoon::pusher::Pusher;
use racoon::{discovery_server::DiscoveryServer, State};
use racoon::{player::RodioPlayer, Api};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let up = fs::read("assets/up.wav").unwrap();
    let down = fs::read("assets/down.wav").unwrap();
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

    #[cfg(debug_assertions)]
    {
        let mut file = File::create("openapi.yml")?;
        file.write_all(api_service.spec_yaml().as_bytes())?;
    }

    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest("/api", api_service)
        .nest("/doc", ui)
        .with(AddData::new(state));

    let _discovery_server = DiscoveryServer::new(port.into());
    let network_interfaces = list_afinet_netifas().unwrap();
    let mut loopback: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let likely_local = local_ip();
    let mut locals = Vec::with_capacity(10);

    let likely_local_qr = if let Ok(ip) = likely_local {
        let code = QrCode::new(&format!("{ip}:{port}")).unwrap();
        code.render::<unicode::Dense1x2>()
            .quiet_zone(true)
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build()
    } else {
        String::from("")
    };

    for (_, ip) in network_interfaces.iter() {
        if ip.is_loopback() {
            loopback = *ip;
        }
        if let IpAddr::V4(ipv4) = ip.to_canonical() {
            if ipv4.is_private() {
                locals.push(ip);
            }
        }
    }

    let mut qr_str = locals
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(",");
    qr_str.push_str(&format!(":{port}"));

    let code = QrCode::new(&qr_str).unwrap();
    let code = code
        .render::<unicode::Dense1x2>()
        .quiet_zone(true)
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    println!("{loopback}",);
    println!("{likely_local_qr}",);
    println!("{code}",);

    Server::new(TcpListener::bind(&format!("0.0.0.0:{port}")))
        .name("racoon")
        .run(app)
        .await
}

/*

*/
