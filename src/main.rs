use rtodo::workspace::Workspace;
use rtodo::cli::{CLI, Commands};
use rtodo::dispatch::{exec_project_cmd, exec_task_cmd};
use clap::Parser;


fn load_workspace() -> Result<Workspace, String> {
    Workspace::load().map_err(|err| format!("error loading the workspace: {err}"))
}

fn save_workspace(workspace: &Workspace) -> Result<(), String> {
   workspace.save().map_err(|err| format!("error saving workspace: {err}"))?;
   Ok(())
}

fn run() -> Result<(), String> {
   let cli = CLI::parse();

   match cli.command {
        Commands::Init => {
            Workspace::init().map_err(|err| format!("Err init the workspace directory: {err}"))?;
        },
        Commands::Project { command } => {
            let mut workspace = load_workspace()?;
            exec_project_cmd(command, &mut workspace)?;
            save_workspace(&workspace)?;
        },
        Commands::Task { command } => {
            let mut workspace = load_workspace()?;
            let project= workspace.active_project().ok_or("There is not active project, please use project set <id> first.")?;
            exec_task_cmd(command, project)?;
            // this could be improve to only save the project for performance.
            save_workspace(&workspace)?;
        }
   };
   Ok(())
}


fn main() {
    if let Err(msg) = run() {
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}
