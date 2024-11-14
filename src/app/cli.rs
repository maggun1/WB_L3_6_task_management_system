use std::{
    thread,
    time::Duration,
    io::{self, Write},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal
};

use crate::manager::task_manager::TaskManager;
use crate::models::task::TaskStatus;

pub fn run_cli(manager: TaskManager) {
    println!("Task management program started. Enter a command or 'exit' to quit.");
    let mut commands_history: Vec<String> = Vec::new();
    let mut history_index = 0;

    terminal::enable_raw_mode().expect("[ERROR]: Failed to enable raw mode");
    'app_loop: loop {
        print!("\r>>> ");
        io::stdout().flush().expect("[ERROR]: Error flushing stdout");

        let mut input = String::new();
        loop {
            if let Event::Key(key_event) = event::read().expect("[ERROR]: Failed to read event") {
                match key_event.code {
                    KeyCode::Enter => {
                        println!();
                        break;
                    }
                    KeyCode::Up => {
                        if history_index > 0 {
                            history_index -= 1;
                        }

                        if let Some(command) = commands_history.get(history_index) {
                            input = command.clone();

                            clear_line();
                            print!(">>> {}", input);
                            io::stdout().flush().unwrap();
                        }
                    }
                    KeyCode::Down => {
                        if history_index + 1 <= commands_history.len() {
                            history_index += 1;
                        }
                        if history_index == commands_history.len() {
                            clear_line();
                            input.clear();
                            print!(">>> {}", input);
                            continue 'app_loop;
                        }

                        if let Some(command) = commands_history.get(history_index) {
                            input = command.clone();

                            clear_line();
                            print!(">>> {}", input);
                            io::stdout().flush().unwrap();
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("{}", c);
                        io::stdout().flush().unwrap();
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        execute!(io::stdout(),
                                cursor::MoveToColumn(0),
                                terminal::Clear(terminal::ClearType::CurrentLine))
                            .unwrap();
                        print!("\r>>> {}", input);
                        io::stdout().flush().unwrap();
                    }
                    _ => {}
                }
            }
        }

        commands_history.push(input.clone());
        history_index = commands_history.len();
        let mut args = input.trim().split_whitespace();
        let command = args.next();

        match command {
            Some("create") => {
                let mut command = String::new();
                while let Some(arg) = args.next() {
                    command.push_str(arg);
                    command.push(' ');
                }
                if command.is_empty() {
                    println!("\rCommand to execute must be specified.");
                }
                else
                {
                    let task_id = manager.create_task(command);
                    println!("\rCreated task with ID: {}", task_id);
                    println!("\rUse `status {}` to check the status", task_id);
                }
            }
            Some("run") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        manager.run_task(id);
                        println!("\rSent run signal for task {}", id);
                    } else {
                        println!("\rInvalid task ID format.");
                    }
                } else {
                    println!("\rTask ID must be specified.");
                }
            }
            Some("stop") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        manager.stop_task(id);
                        println!("\rSent stop signal for task {}", id);

                        if let Some(status) = manager.get_task_status(id) {
                            println!("\rTask {} status: {:?}", id, status);
                        }
                    } else {
                        println!("\rInvalid task ID format!");
                    }
                } else {
                    println!("\rTask ID must be specified.");
                }
            }
            Some("kill") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        manager.kill_task(id);
                        println!("\rSent kill signal for task {}", id);
                    } else {
                        println!("\rInvalid task ID format!");
                    }
                } else {
                    println!("\rTask ID must be specified.");
                }
            }
            Some("status") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        match manager.get_task_status(id) {
                            Some(status) => println!("\rTask {} status: {:?}", id, status),
                            None => println!("\rTask {} not found", id),
                        }
                    } else {
                        println!("\rInvalid task ID format.");
                    }
                } else {
                    println!("\rTask ID must be specified.");
                }
            }
            Some("list") => {
                let tasks = manager.get_all_tasks();
                if tasks.is_empty() {
                    println!("\rNo active tasks");
                } else {
                    println!("\r\tTask list:");
                    println!("\r{}\t {} \t {} \t {}", "ID", "Status", "Command", "PID");
                    println!("\r{}", "-".repeat(60));
                    for task in tasks {
                        let task_pid = if task.pid.is_some() { task.pid.unwrap().to_string() } else { "NONE".to_string() };
                        println!("\r{}\t {:?} \t {} \t {}", task.id, task.status, task.name, task_pid);
                    }
                }
            }
            Some("watch") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        println!("\rWatching task {}.", id);
                        match manager.get_task_status(id) {
                            Some(status) => {
                                println!("\rTask {} status: {:?}", id, status);
                                let task_output = manager.get_task_output(id);
                                for output_line in task_output {
                                    println!("\r{}", output_line);
                                }
                            }
                            None => {
                                println!("\rTask {} not found", id);
                                break;
                            }
                        }
                    } else {
                        println!("\rInvalid task ID format.");
                    }
                } else {
                    println!("\rTask ID must be specified.");
                }
            }
            Some("exit") => {
                println!("\rExiting the program...");
                break;
            }
            Some(cmd) => {
                println!("\rUnknown command: {}. Please try again.", cmd);
            }
            None => continue,
        }
        terminal::disable_raw_mode().expect("[ERROR]:Failed to disable raw mode");
    }

}

fn clear_line() {
    execute!(io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(terminal::ClearType::CurrentLine))
            .expect("[ERROR]: Failed to clear line");
}