mod plot_client;

use packets::PlotKind;
use plot_client::PlotClient;
use std::mem;
use std::net::SocketAddr;

pub struct PlotRemote {
    kind: PlotKind,
    batch_len: usize,
    datum_batch: Vec<(f64, f64)>,
    client: PlotClient,
}

impl PlotRemote {
    pub fn new(plot_server_addr: SocketAddr, kind: PlotKind, batch_len: usize) -> Self {
        Self {
            kind,
            batch_len,
            datum_batch: vec![],
            client: PlotClient::new(plot_server_addr),
        }
    }
    pub fn send(&mut self, datum: (f64, f64)) {
        self.datum_batch.push(datum);
        if self.datum_batch.len() >= self.batch_len {
            let insertion_batch = mem::replace(&mut self.datum_batch, vec![]);
            self.client.send(self.kind, insertion_batch);
        }
    }
}
