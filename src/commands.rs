use crate::models::{Project, Status, Priority, Task};
use std::collections::HashMap;


// this function makes no sense but 
// it is only for learning purposes
fn find_projects_by_name<'a>(projects: &'a[Project], name: &str) -> Result<&'a Project, String> {
    let mut map = HashMap::new();

    for p in projects {
        map.insert(p.name.as_str(), p);
    }

    map.get(name).copied().ok_or(format!("Project {name} not found"))

}


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
    projects: &[Project],
    ) -> Result<(), String> {

    
    let active_project = projects.get(0).expect("Please provide at least one project");


    match command {
        "new" =>  println!("[new] Creating project..."),
        "ls" =>  println!("{}", active_project.summary()),
        "set" => {
            let name = args.get(0).ok_or("Please provide the project name, task foo")?;
            let p = find_projects_by_name(projects, name)?;
            println!("Active project set to {}", p.name);
           },
        "delete" =>  println!("[delete] current project"),
        "task" =>  {
            let second_command = args.get(0).ok_or("Please provide task command, for example: task ls")?;
            match second_command.as_str() {
                "ls" => {
                    if let Some(task) = active_project.active_task() {
                        println!("Current active task: {}", task.summary())
                    }

                    display_tasks_by_status(&active_project, Status::InProgress);
                    display_tasks_by_status(&active_project, Status::New );
                    display_tasks_by_status(&active_project, Status::Completed);
                },
                "set" => {
                    let tid_str = args.get(1).ok_or("Provide the task id: (task set 123)")?;
                    let tid: u32 = tid_str.parse().map_err(|e| format!("Failed to parse tid, {e}"))?;
                    let task = active_project.find_task(tid)?;
                    println!("Active task set to {}", task.id);
                },
                arg => {
                    return Err(format!("Task arg ({arg}) Not yet implemented"));
                }
                }
        }
        _ => {
            return Err(format!("Unknown command ({command})"));
        }
    }

    Ok(())

}
