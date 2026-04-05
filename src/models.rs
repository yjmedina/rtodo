

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

    pub fn add_task(&mut self, description: String, priority: Priority) {
        let id = self.tasks.len() as u32;
        let task = Task::new(id, description, priority,  Some(Status::New));
        self.tasks.push(task);
    }

    pub fn delete_task(&mut self, id: u32) -> Option<Task> {
        // iter finds the position of the index Option<usize>
        let pos = self.tasks.iter().position(|t | t.id == id);

        // if found, then swap it with the last element and return it
        if let Some(i) = pos {
            let task = self.tasks.swap_remove(i);
            Some(task)
        } else {
            None
        }


        // for (i, task ) in self.tasks.iter().enumerate() {
        //     if task.id == id {
        //         let last_item = self.tasks.len() - 1;
        //         self.tasks.swap(i, last_item);
        //         let task =  self.tasks.pop();
        //         return task
        //     }
        // }

        // None
   }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    } 

    pub fn summary(&self) -> String {
        format!("[{}] {} ({} tasks)", self.id, self.name, self.task_count())
    }

    pub fn active_task(&self) -> Option<&Task> {
        if let Some(i) = self.active_task_id {
            self.tasks.iter().find(|t| t.id == i)
        } else {
            None
        }

    //     if let Some(i) = self.active_task_id {
    //         for t in &self.tasks {
    //             if t.id == i {
    //                 return Some(t)
    //             }
    //         }
    //     }
    //     None
    } 

    pub fn find_task(&self, id: u32) -> Result<&Task, String> {
        self.tasks.iter().find(|&t| t.id == id).ok_or(format!("Task with id {id} do not exists"))
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
