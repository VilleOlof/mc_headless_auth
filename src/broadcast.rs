use std::sync::{Arc, Mutex};

use crossbeam::channel::{Receiver, Sender, bounded};

use crate::channel_message::ChannelMessage;

#[derive(Debug, Clone)]
pub(crate) struct Broadcast {
    subs: Arc<Mutex<Vec<Sender<Arc<ChannelMessage>>>>>,
}

impl Broadcast {
    pub fn new() -> Self {
        Self {
            subs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn sub(&self, cap: usize) -> Receiver<Arc<ChannelMessage>> {
        let (s, r) = bounded(cap);
        self.subs.lock().unwrap().push(s);
        r
    }

    pub fn send(&self, msg: ChannelMessage) {
        let msg = Arc::new(msg);

        let mut subs = self.subs.lock().unwrap();
        subs.retain(|s| s.send(msg.clone()).is_ok());
    }
}
