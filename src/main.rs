mod models;
mod worker;
mod app;
mod manager;

use app::cli;
use manager::task_manager::TaskManager;


fn main() {
    let manager = TaskManager::new();
    manager.start();

    cli::run_cli(manager);
}
