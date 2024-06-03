use model::Params;
use packets::{GetParamsReply, LearnerRequest};
use std::net::{SocketAddr, TcpStream};

pub struct LearnerClient {
    server_addr: SocketAddr,
}

impl LearnerClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
    pub fn get_params(&self) -> Params {
        let request = LearnerRequest::GetParams;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to learner: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        let GetParamsReply { params } = tcp_io::deserialize_from(stream).unwrap();
        params
    }
}
