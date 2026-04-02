use std::env;

#[derive(Debug)]
struct Project {
    id: u32,
    name: String,
    tasks: Vec<Task>,
    active_task_id: Option<u32>
}

impl Project {
    fn new(id: u32, name: String) -> Self {
        Project { id, name, tasks: Vec::new(), active_task_id: None}
    }

    fn task_count(&self) -> usize {
        self.tasks.len()
    } 

    fn summary(&self) -> String {
        format!("[{}] {} ({} tasks)", self.id, self.name, self.task_count())
    }

    fn active_task(&self) -> Option<&Task> {
        if let Some(i) = self.active_task_id {
            for t in &self.tasks {
                if t.id == i {
                    return Some(t)
                }
            }
        }
        None
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

    fn sort_order(&self) -> u8 {
        match self {
            Status::InProgress=> 0,
            Status::New => 1,
            Status::Completed => 2,
        }

    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    fn label(&self) -> String {
        let label = match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
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
    priority: Priority,
    status: Status,

}

impl Task {
    fn new(id: u32, description: String, priority: Priority , status: Option<Status>)  -> Self{
        let status = if let Some(s) = status {s} else {Status::New};
        // or  more idiomatic
        // let status = status.unwrap_or(Status::New)
        Task{id, description, priority, status: status}
    }
    fn summary(&self) -> String {
        let marker = if self.priority == Priority::High {"!"} else {" "};
        format!(
            "[{}]{} {} ({}) [{}]", 
            self.id,
            marker,
            self.description,
            self.priority.label(),
            self.status.label(),
        )
    }

}


fn tasks_by_status<'a>(tasks: &'a [Task], status: &Status) -> Vec<&'a Task> {
    let mut filtered_tasks: Vec<&'a Task> = tasks
        .iter()
        .filter(|&t| t.status.sort_order() == status.sort_order())
        .collect();
    filtered_tasks.sort_by(|&a, &b| a.priority.cmp(&b.priority));
    filtered_tasks
}

fn display_tasks_by_status(tasks: &[Task], status: Status, active_task_id: Option<u32>)  {
    println!("[{}]", status.label());

    let ftasks = tasks_by_status(tasks, &status);

    if ftasks.len() == 0 {
        println!("  (none)");

    }

    for t in ftasks {
        print!("  {}", t.summary());
        if let Some(i) = active_task_id && t.id == i {
            print!(" (active)");
        }
        println!()
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
            display_tasks_by_status(&project.tasks, Status::InProgress, project.active_task_id);
            display_tasks_by_status(&project.tasks, Status::New, project.active_task_id);
            display_tasks_by_status(&project.tasks, Status::Completed, project.active_task_id);
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


    let task_1 = Task::new(0, String::from("Learn Rust"), Priority::Medium, Some(Status::Completed));
    let task_2 = Task::new(1, String::from("Implement CLI using rust"), Priority::Medium, Some(Status::InProgress));
    let task_3 = Task::new(2, String::from("Be a millionare"), Priority::Low, None);
    let task_4 = Task::new(3, String::from("Testing task"), Priority::High, Some(Status::InProgress));

    assert_eq!(task_1.status.sort_order(), 2);

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