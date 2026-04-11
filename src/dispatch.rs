use crate::models::{Status, Priority};
use crate::workspace::Workspace;
use crate::cli::{ProjectCommands, TaskCommands};

pub fn exec_cmd(
    command: ProjectCommands,
    workspace: &mut Workspace,
    ) -> Result<(), String> {

    match command {
        ProjectCommands::Add {name} => {
            workspace.add_project(name);
        },
        ProjectCommands::Ls => {
            println!("{}", workspace);
        },
        ProjectCommands::Set {pid } => {
            let p = workspace.set_active_project(pid)?;
            println!("Active project set to '{}'", p);
        },
        ProjectCommands::UnSet => {
            workspace.unset_active_project();
            println!("Active project unset");
        },
        ProjectCommands::Delete { .. } => {
            println!("Not Implemented!");
        }
        ProjectCommands::Task {command} => {
            let active_project = workspace.active_project().ok_or("There is not active project, please use set {name} first.")?;
            match command {
                TaskCommands::Ls => {
                    println!("{}", active_project.task_summary());
                }
                TaskCommands::Add { desc, priority } => {
                let priority = Priority::from(&priority)?;
                // todo add priority
                let task = active_project.add_task(desc, priority);
                println!("Task added succesfully\n{task}");
                },
                TaskCommands::Set { tid } => {
                    let task = active_project.set_active_task(tid)?;
                    println!("Active task: {task}");
                },
                TaskCommands::Completed => {
                    let task = active_project.active_task_completed()?;
                    println!("Completed!: {task}");
                },
                TaskCommands::Move { tid, status } => {
                    let status = Status::from(&status)?;
                    let task = active_project.move_task(tid, status)?;
                    println!("Task moved to {}: {}", task.status, task);
                }


            }
        }

    };
    Ok(())

}