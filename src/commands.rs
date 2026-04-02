use crate::models::{Project, Status};

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


pub fn dispatch(
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
                if let Some(task) = project.active_task() {
                    println!("Current active task: {}", task.summary())
                }

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
