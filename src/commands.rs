use crate::models::{Project, Status, Priority};
use std::collections::HashMap;


// this function makes no sense but 
// it is only for learning purposes
fn find_projects_by_name<'a>(projects: &'a[Project], name: &str) -> Result<&'a Project, String> {
    let mut map = HashMap::new();

    for p in projects {
        map.insert(p.name.as_str(), p);
    }

    map.get(name).copied().ok_or(format!("Project '{name}' not found"))

}


fn display_tasks_by_status(project: &Project, status: Status)  {
    println!("[{}]", status);

    let ftasks = project.tasks_by_status(&status);

    if ftasks.is_empty() {
        println!("  (none)");
        return 
    }

    for t in ftasks {
        println!("{t}");
    }
}

pub fn dispatch(
    command: &str,
    args: &[String],
    projects: &mut [Project],
    ) -> Result<(), String> {

    
    let active_project = projects.get_mut(0).ok_or("Please provide at least one project")?;


    match command {
        "new" =>  println!("[new] Creating project..."),
        "ls" =>  {
            for p in projects {
                println!("{p}");
            }
        }
        "set" => {
            let name = args.get(0).ok_or("Please provide the project name, task foo")?;
            let p = find_projects_by_name(projects, name)?;
            println!("Active project set to '{}'", p.name);
           },
        "delete" =>  println!("[delete] current project"),
        "task" =>  {
            let second_command = args.get(0).ok_or("Please provide task command, for example: task ls")?;
            match second_command.as_str() {
                "ls" => {
                    if let Some(task) = active_project.active_task() {
                        println!("Current active task: {task}")
                    }

                    display_tasks_by_status(&active_project, Status::InProgress);
                    display_tasks_by_status(&active_project, Status::New );
                    display_tasks_by_status(&active_project, Status::Completed);
                },
                "set" => {
                    let tid_str = args.get(1).ok_or("Provide the task id: (task set 123)")?;
                    let tid: u32 = tid_str.parse().map_err(|e| format!("Failed to parse tid, {e}"))?;
                    let task = active_project.find_task(tid)?;
                    println!("Active task set to {task}");
                    active_project.active_task_id = Some(task.id);
                },
                "add" => {
                    let description = args.get(1).ok_or("Provide a description")?.clone();
                    let priority = match args.get(2 ) {
                        Some(s) => Priority::from(s)?,
                        None => Priority::Low,
                    };

                    // todo add priority
                    let task = active_project.add_task(description, priority);
                    println!("Task added succesfully\n{task}");
                    
                },
                "completed" => {
                    let active_task_id = active_project.active_task_id.ok_or("There is not a actice task rightnow, use task set {id}")?;
                    let task = active_project.find_task(active_task_id)?;
                    task.status = Status::Completed;
                    println!("{task}") ;
                }
                "move" => {
                    let task_id_str = args.get(1).ok_or("Provide the task id")?;
                    let task_id: u32 = task_id_str.parse().map_err(|e| format!("Failed to parse task id: {e}"))?;
                    let status= match args.get(2) {
                        Some(s) => Status::from(s)?,
                        None => Status::New
                    };
                    let task = active_project.find_task(task_id)?;
                    task.status = status;
                    println!("Task moved {task}") ;
                }, 
                "delete" => {
                    let task_id_str = args.get(1).ok_or("Provide the task id")?;
                    let task_id: u32 = task_id_str.parse().map_err(|e| format!("Failed to parse task id: {e}"))?;
                    let task = active_project.delete_task(task_id).ok_or(format!("task id {task_id} do not exists"))?;
                    println!("Delete task: {task}");

                }
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
