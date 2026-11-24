//! Wisp development-only remote_control. Explicit runtime opt-in is also required.
pub use crate::capture;
pub mod input;
pub mod protocol;
pub mod server;
pub mod video;

use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, SyncSender},
};
use std::time::Instant;

pub struct RemoteControl {
    pub incoming: Receiver<server::Event>,
    pub session: Option<u64>,
    pub alive: Option<Arc<AtomicBool>>,
    pub input: input::InputState,
    pub pending: Vec<(u64, SyncSender<protocol::Reply>)>,
    pub pointers: BTreeMap<u32, [f64; 2]>,
    pub taps: Vec<([f64; 2], Instant)>,
}
impl RemoteControl {
    pub fn from_env() -> Result<Option<Self>, std::io::Error> {
        if std::env::var("WISP_REMOTE_CONTROL").as_deref() != Ok("1") {
            return Ok(None);
        }
        Ok(Some(Self {
            incoming: server::start()?,
            session: None,
            alive: None,
            input: Default::default(),
            pending: Vec::new(),
            pointers: BTreeMap::new(),
            taps: Vec::new(),
        }))
    }
    pub fn active(&self) -> bool {
        self.session.is_some()
            && self
                .alive
                .as_ref()
                .is_some_and(|a| a.load(Ordering::SeqCst))
    }
    pub fn revoke(&self) {
        if let Some(a) = &self.alive {
            a.store(false, Ordering::SeqCst);
        }
    }
}
