use std::{
    io::{self, Write},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal
};

use crate::manager::task_manager::TaskManager;

pub fn run_cli(manager: TaskManager) {
    println!("Task management system started. Enter 'help' for usage.");
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
                    println!("\rCreated task with ID: '{}'.", task_id);
                    println!("\rUse `status {}` to check the status.", task_id);
                }
            }
            Some("run") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        if !manager.run_task(id) {
                            println!("\rTask with ID '{}' not found.", id);
                            continue;
                        }
                        println!("\rTask with ID '{}' started.", id);
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
                        if !manager.stop_task(id) {
                            println!("\rTask with ID '{}' not found.", id);
                            continue;
                        }
                        println!("\rSent stop signal for task with ID: '{}'.", id);

                        if let Some(status) = manager.get_task_status(id) {
                            println!("\rTask with ID '{}' status: {:?}.", id, status);
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
                        if !manager.kill_task(id) {
                            println!("\rTask with ID '{}' not found.", id);
                            continue;
                        }
                        println!("\rSent kill signal for task with ID: '{}'.", id);
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
                            Some(status) => println!("\rTask with ID '{}' status: {:?}.", id, status),
                            None => println!("\rTask with ID '{}' not found.", id),
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
                    println!("\rNo tasks.");
                } else {
                    println!("\r\tTask list:");
                    println!("\r{}\t {} \t {} \t {}", "ID", "Status", "Command", "PID");
                    println!("\r{}", "-".repeat(55));
                    for task in tasks {
                        let task_pid = if task.pid.is_some() { task.pid.unwrap().to_string() } else { "NONE".to_string() };
                        println!("\r{}\t {:?} \t {} \t {}", task.id, task.status, task.name, task_pid);
                    }
                }
            }
            Some("watch") => {
                if let Some(id_str) = args.next() {
                    if let Ok(id) = id_str.parse::<u64>() {
                        println!("\rWatching task with ID: '{}'.", id);
                        match manager.get_task_status(id) {
                            Some(status) => {
                                println!("\rStatus: {:?}", status);
                                println!("\rOutput:");
                                let task_output = manager.get_task_output(id);
                                for output_line in task_output {
                                    println!("\r{}", output_line);
                                }
                            }
                            None => {
                                println!("\rTask with ID '{}' not found.", id);
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
            Some("help") => {
                println!("\rUsage:");
                println!("\rcreate <command> - Create a new task with the specified command.");
                println!("\rrun <task_id> - Run the task with the specified ID.");
                println!("\rstop <task_id> - Stop the task with the specified ID.");
                println!("\rkill <task_id> - Kill the task with the specified ID.");
                println!("\rstatus <task_id> - Show the status of the task with the specified ID.");
                println!("\rlist - List all active tasks.");
                println!("\rwatch <task_id> - Watch the output of the task with the specified ID.");
                println!("\rquit - Exit the program.");
            }
            Some("quit") => {
                println!("\rExiting the program...");
                break;
            }
            Some(cmd) => {
                println!("\rUnknown command: {}. Please try again.", cmd);
            }
            None => continue,
        }
    }
    terminal::disable_raw_mode().expect("[ERROR]: Failed to disable raw mode");
}

fn clear_line() {
    execute!(io::stdout(),
            cursor::MoveToColumn(0),
            terminal::Clear(terminal::ClearType::CurrentLine))
            .expect("[ERROR]: Failed to clear line");
}