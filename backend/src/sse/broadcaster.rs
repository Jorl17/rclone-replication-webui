use dashmap::DashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 256;

/// Événement diffusé sur le canal SSE par-tâche.
///
/// Les logs ligne par ligne sont stockés dans `RunningTask.log_buffer` (accessible aux clients
/// qui se connectent en cours d'exécution). Le broadcaster ne sert plus qu'à signaler la fin.
#[derive(Debug, Clone)]
pub enum SseEvent {
    Done { status: String, exit_code: Option<i32>, duration_ms: i64 },
}

#[derive(Debug)]
pub struct SseBroadcaster {
    channels: DashMap<Uuid, broadcast::Sender<SseEvent>>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        SseBroadcaster {
            channels: DashMap::new(),
        }
    }

    pub fn subscribe(&self, task_id: Uuid) -> broadcast::Receiver<SseEvent> {
        if let Some(sender) = self.channels.get(&task_id) {
            return sender.subscribe();
        }
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        self.channels.insert(task_id, tx);
        rx
    }

    pub fn publish(&self, task_id: Uuid, event: SseEvent) {
        if let Some(sender) = self.channels.get(&task_id) {
            let _ = sender.send(event);
        }
    }

    pub fn close(&self, task_id: &Uuid) {
        self.channels.remove(task_id);
    }
}
