use std::env;


fn main() {
    println!("rtodo - v0.1.0 - your local task manager");
    let env_args: Vec<String> = env::args().collect();

    if env_args.len() > 1 {
        println!("Command <{}>", env_args[1]);
    } else {
        println!("Usage: rtodo <command>");
    }
}