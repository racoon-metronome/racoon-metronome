use std::{
    net::UdpSocket,
    thread::{self, JoinHandle},
};

fn find_available_port(start: u16, end: u16) -> Option<UdpSocket> {
    for port in start..=end {
        if let Ok(l) = UdpSocket::bind(&format!("0.0.0.0:{port}")) {
            return Some(l);
        }
    }
    None
}

pub struct DiscoveryServer {
    pub _thread: JoinHandle<()>,
}

impl DiscoveryServer {
    pub fn new(port: usize) -> Option<Self> {
        let p = port.to_le_bytes();
        let socket = find_available_port(15987, 16000)?;
        let thread = thread::spawn(move || {
            //FIXME get free ports etc
            // let socket = UdpSocket::bind(&format!("0.0.0.0:{d_port}")).unwrap();
            socket.set_broadcast(true).unwrap();
            loop {
                let mut buf = [0; 3];
                let (_amt, src) = socket.recv_from(&mut buf).unwrap();
                socket.send_to(&p, src).unwrap();
                println!("->> discovery_request - {src}",);
            }
        });
        Some(DiscoveryServer { _thread: thread })
    }
}
