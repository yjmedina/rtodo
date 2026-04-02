use std::env;


fn main() {
    println!("rtodo - v0.1.0 - your local task manager");
    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        let command = env_args[1].as_str();

        if command == "new" {
            println!("[new] create project");
        } else if command == "ls" {
            println!("[list] all projects")
        } else if command == "set" {
            println!("[set] set active project")
        } else if command == "delete" {
            println!("[delete] current project")

        } else if command == "task" {
            println!("[task] exploring some task")
        } else {
            println!("Unknown command: <{command}>")
        }


    } else {
        println!("Usage: rtodo <command>");
    }
}