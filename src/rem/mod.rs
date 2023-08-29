pub mod server;

mod date;
mod http;

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::time::Instant;
use tracing::trace;
use xitca_unsafe_collection::no_hash::NoHashBuilder;

#[derive(Clone)]
pub struct SharedState {
    inner: Arc<Mutex<SharedStateInner>>,
}

struct SharedStateInner {
    last_tick: Instant,
    latencies: HashMap<SocketAddr, Latency, NoHashBuilder>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SharedStateInner {
                last_tick: Instant::now(),
                latencies: HashMap::default(),
            })),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Latency {
    pub total: Duration,
    pub round: u32,
}

impl SharedState {
    pub(super) fn tick(&self, instant: Instant) {
        trace!("Updating tick at {:?}", instant);
        self.inner.lock().last_tick = instant;
    }

    pub(super) fn collect(&self) -> Latencies {
        let latencies = self
            .inner
            .lock()
            .latencies
            .iter()
            .map(|(addr, latency)| {
                (
                    addr.to_string(),
                    format!("{:?}", latency.total / latency.round),
                    latency.round,
                )
            })
            .collect();

        Latencies { latencies }
    }

    pub(super) fn clear(&self) {
        self.inner.lock().latencies.clear();
    }

    pub(super) fn update_average(&self, addr: SocketAddr) {
        let mut inner = self.inner.lock();

        let elapsed = inner.last_tick.elapsed();
        match inner.latencies.get_mut(&addr) {
            Some(latency) => {
                latency.round += 1;
                latency.total += elapsed;

                trace!(
                    "Updating average latency for {:?}. New value: {:?}",
                    addr,
                    latency.total / latency.round
                );
            }
            None => {
                inner.latencies.insert(
                    addr,
                    Latency {
                        total: Duration::from_nanos(0),
                        round: 0,
                    },
                );
            }
        };
    }
}

#[derive(sailfish::TemplateOnce)]
#[template(path = "latency.stpl")]
pub struct Latencies {
    latencies: Vec<(String, String, u32)>,
}
