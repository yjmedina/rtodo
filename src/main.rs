mod models;
mod commands;
use std::env;
use models::{Project, Priority};
use commands::dispatch;
use std::collections::HashMap;


// this function makes no sense but 
// it is only for learning purposes
fn find_projects_by_name<'a>(projects: &'a[Project], name: &str) -> Option<&'a Project> {
    let mut map = HashMap::new();

    for p in projects {
        map.insert(p.name.as_str(), p);
    }

    map.get(name).copied()

}


fn main() {

    let mut project = Project::new(0, String::from("Build rtodo CLI"));


    project.add_task( String::from("Learn Rust"), Priority::Medium);
    project.add_task(String::from("Implement CLI using rust"), Priority::Medium);
    project.add_task( String::from("Be a millionare"), Priority::Low);
    project.add_task( String::from("Testing task"), Priority::High);
    project.add_task( String::from("This will be deleted"), Priority::High);
    project.active_task_id = Some(2);

    let task = project.delete_task(4);
    let projects = [project];

    let active_project = find_projects_by_name(&projects, "Build rtodo CLI").expect("The project do not exists");
    assert!(task.is_some());
    assert_eq!(task.unwrap().description, "This will be deleted");
    


    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        println!("rtodo v0.1.0 — your local task manager");
        let command: &str = &env_args[1];
        let args: &[String]  = &env_args[2..];
        dispatch(command, args, active_project);

    } else {
        println!("Usage: rtodo <command>");
    }

}