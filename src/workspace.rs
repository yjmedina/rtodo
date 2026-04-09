use std::env;
use serde::{Serialize, Deserialize};
use std::path::{self, PathBuf};
use std::fs::File;
use std::error;
use std::io::{BufReader, BufWriter};
use crate::models::Project;


type DynError = Box<dyn std::error::Error>;
const STATE_JSON_PATH: &'static str = ".rtodo/state.json";

// WORKSPACE
#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {

    // ignore the path when writing to disk
    #[serde(skip)]
    path: PathBuf,
    pub projects: Vec<Project>,
    pub active_project_id: Option<u32>
} 


impl Workspace {
    fn new(path: path::PathBuf) -> Self {
        Workspace{projects: Vec::new(), active_project_id: None, path}
    }


    fn load_from_path(path: &path::Path) -> Result<Workspace, Box<dyn error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut workspace: Workspace = serde_json::from_reader(reader)?;
        // since we skip the path, we have to correctly set the file
        workspace.path = path.to_path_buf();
        Ok(workspace)
    }

    pub fn save(&self) -> Result<(), Box<dyn error::Error>> {
        let file = File::create(&self.path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn find_project(&mut self, id: u32) -> Option<usize> {
        self.projects.iter().position(|p| p.id == id)
    }


    pub fn active_project(&mut self) -> Option<&mut Project> {
        let idx = self.find_project(self.active_project_id?)?;
        Some(&mut self.projects[idx])
    }

    pub fn set_active_project(&mut self, id: u32) -> Result<&mut Project, String> {
        let idx = self.find_project(id).ok_or(format!("{id} do not exists"))?;
        self.active_project_id = Some(id);
        Ok(&mut self.projects[idx])
        }

    pub fn init() -> Result<Self, DynError> {
        let current_dir = std::env::current_dir()?;
        let path = current_dir.join(STATE_JSON_PATH);
        if path.is_file() {
            return Err("file already exists!".into())
        }

        let parent_dir = path.parent().expect("It exists because STATE_JSON_PATH have .rtodo as parent");
        std::fs::create_dir_all(parent_dir)?;
        let workspace = Self::new(path);
        workspace.save()?;
        Ok(workspace)
    }

    fn find_path() -> Option<path::PathBuf>{
        let mut dir= env::current_dir().ok()?;

        loop {
            let path = dir.join(STATE_JSON_PATH);
            if path.is_file() {
                return Some(path);
            }

            // move to parent
            if !dir.pop() {
                // if false, then there is not parent
                break;
            }
        }
        
        None
    }


    pub fn load_or_init() -> Result<Self, DynError> {
        match Self::find_path() {
            Some(p) => Self::load_from_path(&p),
            None => Self::init(),
        }

    }


    pub fn add_project(&mut self, name: String) -> &Project {
        let idx = self.projects.len();
        let p = Project::new(idx as u32, name);
        self.projects.push(p);
        &self.projects[idx]

    }


}
