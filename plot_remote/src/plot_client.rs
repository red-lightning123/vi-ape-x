use packets::{PlotKind, PlotRequest};
use std::net::{SocketAddr, TcpStream};

pub struct PlotClient {
    server_addr: SocketAddr,
}

impl PlotClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
    pub fn send(&mut self, kind: PlotKind, batch: Vec<(f64, f64)>) {
        let request = PlotRequest { kind, batch };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(stream, &request).unwrap();
    }
}
