use crate::models::{Project, Status, Priority, Task};



fn format_task_in_line(task: &Task, is_active: bool) -> String {
    let p_marker = if task.priority == Priority::High {"!"} else {" "};
    let active_label = if is_active {"(active)"} else {""};
    format!("  {:>2} {} {:<22}{}", task.id, p_marker, task.description, active_label)
}


fn display_tasks_by_status(project: &Project, status: Status)  {
    println!("[{}]", status.label());

    let ftasks = project.tasks_by_status(&status);

    if ftasks.is_empty() {
        println!("  (none)");
        return 
    }

    for t in ftasks {
        let is_active =  project.active_task_id.is_some_and(| id | t.id == id);
        let task_str = format_task_in_line(t, is_active);
        println!("{task_str}");
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
