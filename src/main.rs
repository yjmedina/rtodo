use std::env;


// const LOW_PRIORITY: u8 = 1;
// const MEDIUM_PRIORITY: u8 = 2;
// const HIGH_PRIORITY: u8 = 3;



// // static lifetime, the string will live for the entire program
// fn get_priority_label(priority: u8) -> &'static str {
//     if priority == LOW_PRIORITY {
//         "low" 
//     } else if priority == MEDIUM_PRIORITY {
//         "medium"
//     } else if priority == HIGH_PRIORITY{
//         "high"
//     } else {
//         "unknown"
//     }

// }


#[derive(Debug)] // what the hell is this?
struct Task {
    id: u32,
    description: String,
    priority: u8

}

fn dispatch(command: &str, args: &[String]) {
    if command == "new" {
        println!("[new] Creating project...");
    } else if command == "ls" {
        println!("[list] all projects")
    } else if command == "set" {
        println!("[set] set active project")
    } else if command == "delete" {
        println!("[delete] current project")

    } else if command == "task" {
        println!("[task] Running task command...")
    } else {
        println!("Unknown command: {command}")
    }

    println!("Extra args: {}", args.len());

}


fn print_command(command: &str) {
    println!("Dispatching: {command}")
}

fn main() {
    let task_1 = Task{
        id: 0,
        description: String::from("Learn Rust"),
        priority: 2,
    };

    let task_2= Task{
        id: 1,
        description: String::from("Use Rust to build my cli"),
        priority: 3,
    };


    println!("{:#?}", task_1);
    println!("{:#?}", task_2);
    println!("task [{}]({}): {}", task_1.id, task_1.priority, task_1.description);


    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        println!("rtodo v0.1.0 — your local task manager");

        // deep copy form env args
        let command: &str = &env_args[1];
        let args: &[String]  = &env_args[2..];
        // command is move to the print_command, can't access after the exec
        // of the fn
        // create other copy of command
        print_command(command);

        // iterate for all arguments
        for (i, arg) in env_args[1.. ].iter().enumerate() {
            println!("arg[{i}]: {arg}");
        }
        dispatch(command, args);

        println!("You typed: {}", env_args[1]);

    } else {
        println!("Usage: rtodo <command>");
    }

}