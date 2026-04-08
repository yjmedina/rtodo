use std::env;
use rtodo::models::Workspace;
use rtodo::commands::dispatch;
use std::path::Path;

fn main() {


    let path = Path::new(".rtodo/state.json");
    let mut workspace = match Workspace::load(&path) {
        Ok(w) => w,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };


    // let mut project = Project::new(0, String::from("Build rtodo CLI"));


    // project.add_task( String::from("Learn Rust"), Priority::Medium);
    // project.add_task(String::from("Implement CLI using rust"), Priority::Medium);
    // project.add_task( String::from("Be a millionare"), Priority::Low);
    // project.add_task( String::from("Testing task"), Priority::High);
    // project.add_task( String::from("This will be deleted"), Priority::High);
    // project.active_task_id = Some(2);

    // let task = project.delete_task(4);
    // assert!(task.is_some());
    // let mut projects = [project];

    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        println!("rtodo v0.1.0 — your local task manager");
        let command: &str = &env_args[1];
        let args: &[String]  = &env_args[2..];
        if let Err(msg) = dispatch(command, args, &mut workspace) {
            eprintln!("[ERROR]: {}", msg);
            std::process::exit(1);
        }

    } else {
        println!("Usage: rtodo <command>");
    }

}