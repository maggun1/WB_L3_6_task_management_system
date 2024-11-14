use std::{
    collections::HashMap,
    sync::{Arc, Mutex}
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;
use crate::models::task::{Task, TaskStatus};
use crate::models::message::TaskMessage;
use crate::worker::worker::spawn_worker_thread;

pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<u64, Task>>>,
    sender: Sender<TaskMessage>,
    receiver: Receiver<TaskMessage>,
    next_id: Arc<Mutex<u64>>,
}

impl TaskManager {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        TaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver,
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn start(&self) {
        let tasks = Arc::clone(&self.tasks);
        let sender = self.sender.clone();
        let receiver = self.receiver.clone();

        spawn_worker_thread(sender, receiver, tasks);
    }

    pub fn create_task(&self, name: String) -> u64 {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            *next_id += 1;
            *next_id
        };
        self.tasks.lock().unwrap().insert(id, Task::new(id, name));
        info!("Task '{}' created.", id);
        id
    }

    pub fn run_task(&self, id: u64) -> bool {
        if self.tasks.lock().unwrap().get(&id).is_none() {
            return false;
        }
        self.sender.send(TaskMessage::Run(id)).unwrap();
        true
    }

    pub fn stop_task(&self, id: u64) -> bool {
        if self.tasks.lock().unwrap().get(&id).is_none() {
            return false;
        }
        self.sender.send(TaskMessage::Stop(id)).unwrap();
        true
    }

    pub fn kill_task(&self, id: u64) -> bool {
        if self.tasks.lock().unwrap().get(&id).is_none() {
            return false;
        }
        self.sender.send(TaskMessage::Kill(id)).unwrap();
        true
    }

    pub fn get_task_status(&self, id: u64) -> Option<TaskStatus> {
        self.tasks.lock().unwrap().get(&id).map(|task| task.status.clone())
    }

    pub fn get_all_tasks(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }

    pub fn get_task_output(&self, id: u64) -> Vec<String> {
        self.tasks.lock().unwrap().get(&id).map(|task| task.output.clone()).unwrap_or(vec![])
    }
}