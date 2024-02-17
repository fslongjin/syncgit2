use crate::config_parser::TaskId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Event {
    valid: bool,
    id: TaskId,
    timestamp: std::time::Instant,
}

impl Event {
    pub fn new(id: TaskId) -> Event {
        Event {
            valid: true,
            id,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn timestamp(&self) -> std::time::Instant {
        self.timestamp
    }
}

impl Default for Event {
    fn default() -> Self {
        Event {
            valid: false,
            id: TaskId::new(""),
            timestamp: std::time::Instant::now(),
        }
    }
}
