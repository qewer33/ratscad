use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use super::runner;

const DEBOUNCE: Duration = Duration::from_millis(400);

pub enum MeshMsg {
    Started,
    Ready { source: String, bytes: Vec<u8> },
    Failed(String),
}

pub struct BuildCoordinator {
    source_tx: Sender<String>,
    mesh_rx: Receiver<MeshMsg>,
}

impl BuildCoordinator {
    pub fn spawn(binary: PathBuf) -> Self {
        let (source_tx, source_rx) = mpsc::channel::<String>();
        let (mesh_tx, mesh_rx) = mpsc::channel::<MeshMsg>();
        thread::spawn(move || worker(binary, source_rx, mesh_tx));
        Self { source_tx, mesh_rx }
    }

    pub fn submit(&self, source: String) {
        let _ = self.source_tx.send(source);
    }

    pub fn drain(&self) -> Vec<MeshMsg> {
        let mut out = Vec::new();
        loop {
            match self.mesh_rx.try_recv() {
                Ok(msg) => out.push(msg),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        out
    }
}

fn worker(binary: PathBuf, source_rx: Receiver<String>, mesh_tx: Sender<MeshMsg>) {
    let mut pending: Option<(String, Instant)> = None;
    loop {
        let recv_timeout = pending
            .as_ref()
            .map(|(_, t)| DEBOUNCE.saturating_sub(t.elapsed()))
            .unwrap_or(Duration::from_secs(60));

        match source_rx.recv_timeout(recv_timeout) {
            Ok(source) => {
                pending = Some((source, Instant::now()));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some((source, started)) = pending.take()
                    && started.elapsed() >= DEBOUNCE
                {
                    let _ = mesh_tx.send(MeshMsg::Started);
                    match runner::run_openscad(&binary, &source) {
                        Ok(bytes) => {
                            let _ = mesh_tx.send(MeshMsg::Ready { source, bytes });
                        }
                        Err(err) => {
                            let _ = mesh_tx.send(MeshMsg::Failed(err.to_string()));
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}
