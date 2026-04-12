use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init,
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    Add { name: String },
    Ls,
    Set { pid: u32 },
    UnSet,
    Delete { pid: u32 },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    Add {
        desc: String,
        #[arg(short, long, default_value_t = String::from("medium"))]
        priority: String,
    },
    Ls,
    Set {
        tid: u32,
    },
    Completed,
    Move {
        tid: u32,
        status: String,
    },
}
