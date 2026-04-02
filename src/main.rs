mod models;
use std::env;
use models::{Task, Project, Status, Priority};

fn display_tasks_by_status(project: &Project, status: Status)  {
    println!("[{}]", status.label());

    let ftasks = project.tasks_by_status(&status);

    if ftasks.is_empty() {
        println!("  (none)");
        return 
    }

    for t in ftasks {
        print!("  {}", t.summary());
        if let Some(i) = project.active_task_id && t.id == i {
            print!(" (active)");
        }
        println!()
    }
}


fn dispatch(
    command: &str,
    args: &[String],
    project: &Project) {


    match command {
        "new" =>  println!("[new] Creating project..."),
        "ls" =>  println!("{}", project.summary()),
        "set" =>  println!("[set] set active project"),
        "delete" =>  println!("[delete] current project"),
        "task" =>  {
            let second_command = &args[0];

            if second_command == "ls" {
                display_tasks_by_status(&project, Status::InProgress);
                display_tasks_by_status(&project, Status::New );
                display_tasks_by_status(&project, Status::Completed);
            }
            else  {
                println!("[Task {second_command}] Not yet implemented");
                }
        }
        _ => println!("Unknown command: {command}")
    }

}


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

    if let Some(task) = project.active_task() {
        println!("Current active task: {}", task.summary())
    } else {
        println!("No active task!")
    }


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