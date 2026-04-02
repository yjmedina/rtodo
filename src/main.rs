use std::env;


const LOW_PRIORITY: u8 = 1;
const MEDIUM_PRIORITY: u8 = 2;
const HIGH_PRIORITY: u8 = 3;

#[derive(Debug)]
struct Project {
    id: u32,
    name: String,
    tasks: Vec<Task>
}

impl Project {
    fn new(id: u32, name: String) -> Self {
        Project { id, name, tasks: Vec::new() }
    }

    fn task_count(&self) -> usize {
        self.tasks.len()
    } 

    fn summary(&self) -> String {
        format!("{} ({} tasks)", self.name, self.task_count())
    }

}


// Impl the debug trait, which allows to 
// print using {:#?} while using println!
#[derive(Debug)] 
struct Task {
    id: u32,
    description: String,
    priority: u8

}

impl Task {
    fn new(id: u32, description: String, priority: u8)  -> Self{
        Task{id, description, priority}
    }

    fn priority_label(&self) -> &'static str{
        if self.priority == LOW_PRIORITY {
                "low" 
            } else if self.priority == MEDIUM_PRIORITY {
                "medium"
            } else if self.priority == HIGH_PRIORITY{
                "high"
            } else {
                "unknown"
            }
    }

    fn summary(&self) -> String {
        format!(
            "[{}] {} ({})", 
            self.id,
            self.description,
            self.priority_label()
        )

    }

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

    let mut project = Project::new(0, String::from("Build rtodo CLI"));

    let task_1 = Task::new(0, String::from("Learn Rust"), 2);
    let task_2 = Task::new(1, String::from("Implement CLI using rust"), 3);

    project.tasks.push(task_1);
    project.tasks.push(task_2);

    println!("project summary: {}", project.summary());

    for t in &project.tasks{
        println!("{}", t.summary());
    }

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