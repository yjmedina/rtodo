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


fn dispatch(command: &str) {
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

}


fn main() {
    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        println!("rtodo v0.1.0 — your local task manager");
        // iterate for all arguments
        for (i, arg) in env_args[1.. ].iter().enumerate() {
            println!("arg[{i}]: {arg}");
        }

        dispatch(&env_args[1]);

    } else {
        println!("Usage: rtodo <command>");
    }

}