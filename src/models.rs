

// PROJECT
#[derive(Debug)]
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

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    } 

    pub fn summary(&self) -> String {
        format!("[{}] {} ({} tasks)", self.id, self.name, self.task_count())
    }

    pub fn active_task(&self) -> Option<&Task> {
        if let Some(i) = self.active_task_id {
            for t in &self.tasks {
                if t.id == i {
                    return Some(t)
                }
            }
        }
        None
    } 

    pub fn tasks_by_status(&self, status: &Status) -> Vec<&Task> {
        let mut filtered_tasks: Vec<&Task> = self.tasks
            .iter()
            .filter(|&t| t.status == *status)
            .collect();
        filtered_tasks.sort_by_key(|&t| &t.priority);
        filtered_tasks
    }


}




#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status{
    New,
    InProgress,
    Completed
}

impl Status {
    pub fn label(&self) -> String {
        let label = match self {
            Status::New => "new",
            Status::InProgress => "in_progress",
            Status::Completed => "completed",

        };
        String::from(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn label(&self) -> String {
        let label = match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        };
        String::from(label)
    }
}


// Impl the debug trait, which allows to 
// print using {:#?} while using println!
#[derive(Debug)] 
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
    pub fn summary(&self) -> String {
        let marker = if self.priority == Priority::High {"!"} else {" "};
        format!(
            "[{}]{} {} ({}) [{}]", 
            self.id,
            marker,
            self.description,
            self.priority.label(),
            self.status.label(),
        )
    }

}
