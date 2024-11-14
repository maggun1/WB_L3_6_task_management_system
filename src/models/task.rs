#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub status: TaskStatus,
    pub pid: Option<u32>,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Stopped,
    Killed,
}

impl Task {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            status: TaskStatus::Pending,
            pid: None,
            output: vec![],
        }
    }
}