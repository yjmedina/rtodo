use std::env;
use serde::{Serialize, Deserialize};
use std::path;
use std::fs::File;
use std::error;
use std::io::{BufReader, BufWriter};
use crate::models::Project;



const STATE_JSON_PATH: &'static str = ".rtodo/state.json";

// WORKSPACE
#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub projects: Vec<Project>,
    pub active_project_id: Option<u32>
} 


impl Workspace {
    pub fn new() -> Self {
        Workspace{projects: Vec::new(), active_project_id: None}
    }


    pub fn load(path: &path::Path) -> Result<Workspace, Box<dyn error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let workspace: Workspace = serde_json::from_reader(reader)?;
        Ok(workspace)
    }

    pub fn save(&self, path: &path::Path) -> Result<(), Box<dyn error::Error>> {
        let file = File::open(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn find_project(&mut self, id: u32) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.id == id)
    }


    pub fn active_project(&mut self) -> Option<&mut Project> {
        self.find_project(self.active_project_id?)
    }

    pub fn set_active_project(&mut self, id: u32) -> Result<(), String> {
        self.find_project(id).ok_or(format!("{id} do not exists"))?;
        self.active_project_id = Some(id);
        Ok(())
        }

    pub fn find_project_by_name(&self, name: &str) -> Option<& Project> {
        self.projects.iter().find(|t| t.name == name)
    }

}



pub fn find_workspace_path() -> Result<path::PathBuf, Box<dyn std::error::Error>>{
    let mut dir= env::current_dir().expect("msg");

    loop {
        let path = dir.join(STATE_JSON_PATH);
        if path.is_file() {
            return Ok(path);
        }

        // move to parent
        if !dir.pop() {
            // if false, then there is not parent
            break;
        }
    }

    Err("The file do not exists!".into())

}

