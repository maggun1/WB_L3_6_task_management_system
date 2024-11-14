use std::{
    collections::HashMap,
    process::{Command,Stdio},
    sync::{Arc, Mutex},
    thread,
    io::{BufReader,BufRead}
};

use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use libc;

use crate::models::{
    task::{Task, TaskStatus},
    message::TaskMessage
};

pub fn spawn_worker_thread(
    sender: Sender<TaskMessage>,
    receiver: Receiver<TaskMessage>,
    tasks: Arc<Mutex<HashMap<u64, Task>>>)
{
    thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(message) => match message {
                    TaskMessage::Run(id) => handle_run_task(id, &tasks, &sender),
                    TaskMessage::Stop(id) => handle_stop_task(id, &tasks),
                    TaskMessage::Kill(id) => handle_kill_task(id, &tasks),
                    TaskMessage::WriteOutput(id, output) => handle_output_write(id, output, &tasks),
                    TaskMessage::UpdateStatus(id, status) => handle_status_update(id, status, &tasks),
                },
                Err(e) => {
                    error!("Failed to receive message: {}", e);
                    break;
                }
            }
        }
    });
}

fn handle_run_task(
    id: u64,
    tasks: &Arc<Mutex<HashMap<u64, Task>>>,
    sender: &Sender<TaskMessage>)
{
    info!("Starting task '{}'.", id);
    let tasks_guard = tasks.lock().unwrap();
    let mut task = tasks_guard.get(&id).unwrap().clone();
    drop(tasks_guard);

    match Command::new("sh")
        .arg("-c")
        .arg(&task.name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(mut child) => {
            task.pid = Some(child.id());
            task.status = TaskStatus::Running;

            let sender = sender.clone();
            let task_id = task.id;

            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();


            let stdout_sender = sender.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        stdout_sender.send(TaskMessage::WriteOutput(task_id, format!("[stdout] {}", line))).unwrap();
                    }
                }
            });


            let stderr_sender = sender.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        stderr_sender.send(TaskMessage::WriteOutput(task_id, format!("[stderr] {}", line))).unwrap();
                    }
                }
            });


            thread::spawn(move || {
                match child.wait() {
                    Ok(status) => {
                        let new_status = if status.success() {
                            TaskStatus::Completed
                        } else {
                            TaskStatus::Failed
                        };
                        sender.send(TaskMessage::UpdateStatus(task_id, new_status)).unwrap();
                    }
                    Err(e) => {
                        error!("Failed to wait for child process: {}", e);
                        sender.send(TaskMessage::UpdateStatus(task_id, TaskStatus::Failed)).unwrap();
                    }
                }
            });

            tasks.lock().unwrap().insert(task.id, task);
        }
        Err(e) => {
            error!("Failed to run command: {}", e);
            task.status = TaskStatus::Failed;
            tasks.lock().unwrap().insert(task.id, task);
        }
    }
}

fn handle_stop_task(
    id: u64,
    tasks: &Arc<Mutex<HashMap<u64, Task>>>)
{
    let guard = tasks.lock().unwrap();
    if let Some(task) = guard.get(&id) {
        if let Some(pid) = task.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
            drop(guard);
            handle_status_update(id, TaskStatus::Stopped, tasks);
            info!("Task '{}' stopped.", id);
        }
    }
}

fn handle_kill_task(
    id: u64,
    tasks: &Arc<Mutex<HashMap<u64, Task>>>)
{
    let guard = tasks.lock().unwrap();
    if let Some(task) = guard.get(&id) {
        if let Some(pid) = task.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
            drop(guard);
            handle_status_update(id, TaskStatus::Killed, tasks);
            info!("Task '{}' killed.", id);
        }
    }
}

fn handle_status_update(
    id: u64,
    status: TaskStatus, tasks: &Arc<Mutex<HashMap<u64, Task>>>)
{
    if let Some(task) = tasks.lock().unwrap().get_mut(&id) {
        if task.status == TaskStatus::Stopped ||
            task.status == TaskStatus::Killed {
            return;
        }
        task.status = status.clone();
        info!("Task status updated {}: {:?}", id, status);
    }
}

fn handle_output_write(id: u64, output_line: String, tasks: &Arc<Mutex<HashMap<u64, Task>>>) {
    if let Some(task) = tasks.lock().unwrap().get_mut(&id) {
        task.output.push(output_line);
    }
}