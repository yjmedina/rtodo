use std::fmt;
use serde::{Serialize, Deserialize};
use std::fmt::Write;
use std::convert::TryFrom;

// PROJECT
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub tasks: Vec<Task>,
    pub active_task_id: Option<u32>
}

impl Project {
    pub fn new(id: u32, name: String) -> Self {
        Project { id, name, tasks: Vec::new(), active_task_id: None}
    }

    pub fn add_task(&mut self, description: String, priority: Priority) -> &Task {
        let idx = self.tasks.len();
        let task = Task::new(idx as u32, description, priority,  Some(Status::New));
        self.tasks.push(task);
        &self.tasks[idx]
    }

    pub fn delete_task(&mut self, id: u32) -> Option<Task> {
        // iter finds the position of the index Option<usize>
        let pos = self.tasks.iter().position(|t | t.id == id)?;
        Some(self.tasks.swap_remove(pos))
   }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    } 

    pub fn active_task(&self) -> Option<&Task> {
        if let Some(id) = self.active_task_id {
            let i = self.find_task(id)?;
            Some(&self.tasks[i])
        } else {
            None
        }
   } 


   pub fn set_active_task(&mut self, id: u32) -> Result<&mut Task, String> {
        let idx = self.find_task(id).ok_or(format!("Task {id} do not exists"))?;
        self.active_task_id = Some(id);
        let task = &mut self.tasks[idx];
        task.status = Status::InProgress;
        Ok(task)
   }

   pub fn move_task(&mut self, id: u32, status: Status) -> Result<&Task, String> {
        let idx = self.find_task(id).ok_or(format!("Task {id} do not exists"))?;
        self.tasks[idx].status = status;
        Ok(&self.tasks[idx])
   }

   pub fn active_task_completed(&mut self) -> Result<&Task, String> {
        let id = self.active_task_id.ok_or("No active task, please set an active task or use move directly")?;
        self.move_task(id, Status::Completed)

   }


    pub fn find_task(&self, id: u32) -> Option<usize> {
        self.tasks.iter().position(|t| t.id == id)
    } 


    // define lifetimes for practice purposes only
    pub fn tasks_by_status<'a>(&'a self, status: &Status) -> Vec<&'a Task> {
        let mut filtered_tasks: Vec<&Task> = self.tasks
            .iter()
            .filter(|&t| t.status == *status)
            .collect();
        filtered_tasks.sort_by_key(|&t| &t.priority);
        filtered_tasks
    }


    pub fn task_summary(&self) -> String {
        let mut out = String::new();
        for status in &[Status::InProgress, Status::New, Status::Completed] {
            writeln!(out, "{status}").unwrap();

            let ftasks = self.tasks_by_status(status);

            if ftasks.is_empty() {
                writeln!(out, "  (none)").unwrap();
                continue;
            }

            for t in ftasks {
                writeln!(out, "  {t}").unwrap();
            }
        }

        out
    }
}


impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {} ({} tasks)", self.id, self.name, self.task_count())
    }
}



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status{
    New,
    InProgress,
    Completed
}

impl TryFrom<&str> for Status {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "completed" => Ok(Status::Completed),
            "in_progress" => Ok(Status::InProgress),
            "new" => Ok(Status::New),
            _ => Err(format!("unknown status: {s}, allowed options [completed, in_progress, new]")),
        }

    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self {
            Status::New => write!(f, "new"),
            Status::InProgress => write!(f, "in_progress"),
            Status::Completed => write!(f, "completed"),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl TryFrom<&str> for Priority {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error>{
        match s {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            _ => Err(format!("unknown priority {s}, allowed options [low, medium, high]")) 
        }
    }
    
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}



// Impl the debug trait, which allows to 
// print using {:#?} while using println!
#[derive(Debug, Serialize, Deserialize)] 
pub struct Task {
    pub id: u32,
    pub description: String,
    pub priority: Priority,
    pub status: Status,

}

impl Task {
    pub fn new(id: u32, description: String, priority: Priority , status: Option<Status>)  -> Self{
        let status = if let Some(s) = status {s} else {Status::New};
        // or  more idiomatic
        // let status = status.unwrap_or(Status::New)
        Task{id, description, priority, status: status}
    }
}


impl fmt::Display for Task {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let marker = if self.priority == Priority::High {"!"} else {" "};
        write!(
            f,
            "[{}]{} {} ({}) [{}]", 
            self.id,
            marker,
            self.description,
            self.priority,
            self.status,
        )
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    fn get_project() -> Project {
        Project::new(0, String::from("A testing project"))
    }

    #[test]
    fn add_task_increments_id() {
        let mut project = get_project();
        let first_task = project.add_task(String::from("My first task"), Priority::Low);
        assert_eq!(first_task.id, 0);
        let second_task = project.add_task(String::from("My second task"), Priority::Low);
        assert_eq!(second_task.id, 1);
    }

    #[test]
    fn delete_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        let deleted_task = project.delete_task(0).expect("Task 0 should exists");
        assert_eq!(deleted_task.id, 0);
        assert_eq!(&deleted_task.description, "My first task");
        assert_eq!(project.task_count(), 0);
    }

    #[test]
    fn delete_missing_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        let deleted_task = project.delete_task(99);
        assert!(deleted_task.is_none(), "the task with id 99 must no exists");
        assert_eq!(project.task_count(), 1);
    }

    #[test]
    fn find_active_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        project.active_task_id = Some(1);
        let task =  project.active_task().expect("Active task must be the second task");
        assert_eq!(task.id, 1);
        assert_eq!(task.description, "My Second task");
    }

    #[test]
    fn find_active_task_is_none() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        let task =  project.active_task();
        assert!(task.is_none());
    }

    #[test]
    fn find_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        let idx =  project.find_task(0).expect("Task 0 must exists");
        assert_eq!(idx, 0);
        assert_eq!(&project.tasks[idx].description, "My first task");
    }

    #[test]
    fn find_missing_task() {
        let mut project = get_project();
        project.add_task(String::from("My first task"), Priority::Low);
        project.add_task(String::from("My Second task"), Priority::Low);
        let task =  project.find_task(99);
        assert!(task.is_none());
    }




}