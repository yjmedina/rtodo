mod models;
mod commands;
use std::env;
use models::{Task, Project, Status, Priority};
use commands::dispatch;


fn main() {

    let mut project = Project::new(0, String::from("Build rtodo CLI"));


    let task_1 = Task::new(0, String::from("Learn Rust"), Priority::Medium, Some(Status::Completed));
    let task_2 = Task::new(1, String::from("Implement CLI using rust"), Priority::Medium, Some(Status::InProgress));
    let task_3 = Task::new(2, String::from("Be a millionare"), Priority::Low, None);
    let task_4 = Task::new(3, String::from("Testing task"), Priority::High, Some(Status::InProgress));


    project.tasks.push(task_1);
    project.tasks.push(task_2);
    project.tasks.push(task_3);
    project.tasks.push(task_4);
    project.active_task_id = Some(2);
    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        println!("rtodo v0.1.0 — your local task manager");
        let command: &str = &env_args[1];
        let args: &[String]  = &env_args[2..];
        dispatch(command, args, &project);

    } else {
        println!("Usage: rtodo <command>");
    }

}