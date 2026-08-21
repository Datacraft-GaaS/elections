use std::sync::Arc;
use tokio::sync::broadcast;

pub const SHARD_COUNT: usize = 16;
pub const CHANNEL_CAPACITY: usize = 1024;

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub station_code: String,
    pub payload: String,
}

pub struct ShardedBroadcast {
    shards: Vec<broadcast::Sender<SseEvent>>,
}

impl ShardedBroadcast {
    pub fn new() -> Arc<Self> {
        let shards = (0..SHARD_COUNT)
            .map(|_| broadcast::channel::<SseEvent>(CHANNEL_CAPACITY).0)
            .collect();
        Arc::new(Self { shards })
    }

    pub fn shard_for(&self, station_code: &str) -> &broadcast::Sender<SseEvent> {
        let idx = station_shard(station_code);
        &self.shards[idx]
    }

    pub fn subscribe(&self, station_code: &str) -> broadcast::Receiver<SseEvent> {
        self.shard_for(station_code).subscribe()
    }
}

impl Default for ShardedBroadcast {
    fn default() -> Self {
        let shards = (0..SHARD_COUNT)
            .map(|_| broadcast::channel::<SseEvent>(CHANNEL_CAPACITY).0)
            .collect();
        Self { shards }
    }
}

pub fn station_shard(station_code: &str) -> usize {
    let prefix = station_code.as_bytes().first().copied().unwrap_or(0);
    (prefix as usize) % SHARD_COUNT
}
