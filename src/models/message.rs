use super::task::TaskStatus;

#[derive(Debug)]
pub enum TaskMessage {
    Run(u64),
    Stop(u64),
    Kill(u64),
    UpdateStatus(u64, TaskStatus),
    WriteOutput(u64, String),
}