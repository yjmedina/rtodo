use crate::models::{Project, Status, Priority, Task};
use std::collections::HashMap;


// this function makes no sense but 
// it is only for learning purposes
fn find_projects_by_name<'a>(projects: &'a[Project], name: &str) -> Result<&'a Project, String> {
    let mut map = HashMap::new();

    for p in projects {
        map.insert(p.name.as_str(), p);
    }

    match map.get(name).copied() {
        Some(p) => Ok(p),
        None => Err(format!("Project {name} not found"))
    }

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
    ) {

    
    let active_project = &projects[0];


    match command {
        "new" =>  println!("[new] Creating project..."),
        "ls" =>  println!("{}", active_project.summary()),
        "set" => {
            let name = args.get(0);

            if let Some(n) = name {
                match find_projects_by_name(projects, n) {
                    Ok(p) => println!("Active project set to {}", p.name),
                    Err(msg) => println!("[ERROR] {msg}")
                }
            } else {
                println!("[ERROR] please provide the name of the project as set project1")
            }
            } 
        "delete" =>  println!("[delete] current project"),
        "task" =>  {
            let second_command: Option<&str> = args.get(0).and_then(|arg| Some(arg.as_str()));

            match second_command {
                Some("ls") => {
                    if let Some(task) = active_project.active_task() {
                        println!("Current active task: {}", task.summary())
                    }

                    display_tasks_by_status(&active_project, Status::InProgress);
                    display_tasks_by_status(&active_project, Status::New );
                    display_tasks_by_status(&active_project, Status::Completed);
                },
                Some("set") => {
                    if let Some(tid_str) = args.get(1) {
                        match tid_str.parse() {
                            Ok(tid) => {
                                match active_project.find_task(tid) {
                                    Ok(t) => println!("Active task set to {}", t.id),
                                    Err(msg) => println!("[ERROR] {msg}")
                                }

                            }
                            Err(msg) => println!("[ERROR] Failed to parse tid, {msg}")
                        }

                    } else {
                        println!("[ERROR] Provide the task id: (tasl set 123)")
                    }

                }
                Some(arg) => println!("[ERROR Task {arg}] Not yet implemented"),
                None => println!("[ERROR] Use task ls for example")
                }
        }
        _ => println!("Unknown command: {command}")
    }

}
