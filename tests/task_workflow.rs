use rtodo::models::{Priority, Project};

#[test]
fn find_activate_task_and_simulate_completed() {
    let mut project = Project::new(0, String::from("A testing project"));
    project.add_task(String::from("My first task"), Priority::Low);
    project.add_task(String::from("My Second task"), Priority::Low);
    project.active_task_id = Some(1);
    let task = project
        .active_task()
        .expect("Active task must be the second task");
    assert_eq!(task.id, 1);
    assert_eq!(task.description, "My Second task");

    project.active_task_id = None;

    let task = project.active_task();
    assert!(task.is_err());
}
