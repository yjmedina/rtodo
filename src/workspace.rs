//! Workspace persistence for `rtodo`.
//!
//! A workspace is stored as `.rtodo/state.json` somewhere in the directory
//! tree. [`Workspace::init`] creates it in the current directory;
//! [`Workspace::load`] walks parent directories to find the nearest one.

use crate::models::Project;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::env;
use std::error;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{self, PathBuf};

type DynError = Box<dyn std::error::Error>;
const STATE_JSON_PATH: &str = ".rtodo/state.json";

/// Persistent state for a single `rtodo` workspace.
///
/// The `path` field points to the on-disk `state.json` file and is excluded
/// from serialization — it is re-populated on load.
#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    /// Resolved path to `state.json` on disk (not serialized).
    #[serde(skip)]
    path: PathBuf,
    /// All projects tracked by this workspace.
    pub projects: Vec<Project>,
    /// ID of the currently active project, if any.
    pub active_project_id: Option<u32>,
}

impl Workspace {
    /// Create an empty workspace that will be saved to `path`.
    fn new(path: path::PathBuf) -> Self {
        Workspace {
            projects: Vec::new(),
            active_project_id: None,
            path,
        }
    }

    /// Deserialize a workspace from `path` and restore the skipped `path` field.
    fn load_from_path(path: &path::Path) -> Result<Workspace, Box<dyn error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut workspace: Workspace = serde_json::from_reader(reader)?;
        // `path` is skipped during deserialization, so we restore it manually.
        workspace.path = path.to_path_buf();
        Ok(workspace)
    }

    /// Serialize this workspace to its `state.json` file.
    ///
    /// # Errors
    /// Returns `Err` if the file cannot be created or written.
    pub fn save(&self) -> Result<(), Box<dyn error::Error>> {
        let file = File::create(&self.path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Return the index of the project with `id` within `self.projects`, or
    /// `None` if not found.
    pub fn find_project(&self, id: u32) -> Option<usize> {
        self.projects.iter().position(|p| p.id == id)
    }

    /// Return a mutable reference to the active project, or `None` if no
    /// project is currently active.
    pub fn active_project(&mut self) -> Option<&mut Project> {
        let idx = self.find_project(self.active_project_id?)?;
        Some(&mut self.projects[idx])
    }

    /// Set project `id` as the active project.
    ///
    /// # Errors
    /// Returns `Err` if no project with `id` exists in the workspace.
    pub fn set_active_project(&mut self, id: u32) -> Result<&mut Project, String> {
        let idx = self
            .find_project(id)
            .ok_or_else(|| format!("Project {id} not found."))?;
        self.active_project_id = Some(id);
        Ok(&mut self.projects[idx])
    }

    /// Clear the active project selection.
    pub fn unset_active_project(&mut self) {
        self.active_project_id = None;
    }

    /// Initialize a new workspace in the current directory.
    ///
    /// Creates `.rtodo/state.json` and returns the new workspace.
    ///
    /// # Errors
    /// Returns `Err` if the workspace already exists or the file cannot be created.
    pub fn init() -> Result<Self, DynError> {
        let current_dir = std::env::current_dir()?;
        let path = current_dir.join(STATE_JSON_PATH);
        if path.is_file() {
            return Err("Workspace already initialized in this directory.".into());
        }

        let parent_dir = path
            .parent()
            // Safety: always `Some` because STATE_JSON_PATH has `.rtodo` as a parent component.
            .expect("STATE_JSON_PATH must have a parent directory");
        std::fs::create_dir_all(parent_dir)?;
        let workspace = Self::new(path);
        workspace.save()?;
        Ok(workspace)
    }

    /// Walk parent directories from the current working directory to find
    /// the nearest `.rtodo/state.json`.
    fn find_path() -> Option<path::PathBuf> {
        let mut dir = env::current_dir().ok()?;

        loop {
            let path = dir.join(STATE_JSON_PATH);
            if path.is_file() {
                return Some(path);
            }

            // Advance to parent; `pop` returns `false` when at the filesystem root.
            if !dir.pop() {
                break;
            }
        }

        None
    }

    /// Discover and load the nearest workspace.
    ///
    /// # Errors
    /// Returns `Err` if no `.rtodo/state.json` is found in the directory tree
    /// or the file cannot be parsed.
    pub fn load() -> Result<Self, DynError> {
        let p = Self::find_path()
            .ok_or("No workspace found. Run `rtodo init` in your project directory.")?;
        Self::load_from_path(&p)
    }

    /// Append a new project and return a reference to it.
    pub fn add_project(&mut self, name: String) -> &Project {
        let idx = self.projects.len();
        let p = Project::new(idx as u32, name);
        self.projects.push(p);
        &self.projects[idx]
    }

    /// Delete a project
    pub fn delete_project(&mut self, id: u32) -> Result<Project, String> {
        let idx = self
            .find_project(id)
            .ok_or_else(|| format!("Project {id} not found"))?;
        // reset active project id
        if self.active_project_id == Some(id) {
            self.active_project_id = None
        }
        Ok(self.projects.swap_remove(idx))
    }

    /// Delete a project
    pub fn edit_project(&mut self, id: u32, name: String) -> Result<&Project, String> {
        let idx = self
            .find_project(id)
            .ok_or_else(|| format!("Project {id} not found"))?;
        self.projects[idx].name = name;
        Ok(&self.projects[idx])
    }
}

impl Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for p in &self.projects {
            let active_label = if self.active_project_id == Some(p.id) {
                "[ACTIVE]"
            } else {
                ""
            };
            writeln!(f, "{} {}", p, active_label)?;
        }
        Ok(())
    }
}
