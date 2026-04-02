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
        format!("[{}] {} ({} tasks)", self.id, self.name, self.task_count())
    }

}




#[derive(Debug)] 
enum Status{
    New,
    InProgress,
    Completed
}


impl Status {
    fn label(&self) -> String {
        let label = match self {
            Status::New => "new",
            Status::InProgress => "in_progress",
            Status::Completed => "completed",

        };
        String::from(label)
    }
}

// Impl the debug trait, which allows to 
// print using {:#?} while using println!
#[derive(Debug)] 
struct Task {
    id: u32,
    description: String,
    priority: u8,
    status: Status,

}

impl Task {
    fn new(id: u32, description: String, priority: u8, status: Option<Status>)  -> Self{

        let status = if let Some(s) = status {s} else {Status::New};
        // or  more idiomatic
        // let status = status.unwrap_or(Status::New)
        Task{id, description, priority, status: status}
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
            "[{}] {} ({}) [{}]", 
            self.id,
            self.description,
            self.priority_label(),
            self.status.label(),
        )

    }

}


fn dispatch(
    command: &str,
    args: &[String],
    project: &Project) {
    if command == "new" {
        println!("[new] Creating project...");
    } else if command == "ls" {
        println!("{}", project.summary());
    } else if command == "set" {
        println!("[set] set active project")
    } else if command == "delete" {
        println!("[delete] current project")

    } else if command == "task" {
        let second_command = &args[0];
        if second_command == "ls" {
            for t in &project.tasks {
                // print summary with two pace indentation
                println!("  {}", t.summary())
            }
        }
        else  {
            println!("[Task {second_command}] Not yet implemented");
        }
    } else {
        println!("Unknown command: {command}")
    }

}

fn main() {

    let mut project = Project::new(0, String::from("Build rtodo CLI"));

    let task_1 = Task::new(0, String::from("Learn Rust"), 2, Some(Status::Completed));
    let task_2 = Task::new(1, String::from("Implement CLI using rust"), 3, Some(Status::InProgress));

    project.tasks.push(task_1);
    project.tasks.push(task_2);
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