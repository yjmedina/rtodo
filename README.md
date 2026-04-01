# rtodo — Todo CLI in Rust

A small CLI to manage tasks organized by projects, local to your working directory.

---

## Ubiquitous Language

These terms have precise meanings throughout the codebase, commands, and documentation.

| Term | Definition |
|---|---|
| **Workspace** | A directory tree containing a `.rtodo/state.json` file. Discovered by walking up from the current working directory, like `git`. All commands operate on the nearest workspace found. |
| **Project** | A named container for tasks within a workspace. Identified by a unique name and an internal integer ID. Every command that operates on tasks requires an active project. |
| **Active Project** | The single project currently selected for work. All `task` subcommands implicitly target the active project. Stored in `state.json`. |
| **Task** | A unit of work belonging to a project. Has a description, a status, a priority, and optionally the active marker. |
| **Task ID** | A sequential integer assigned per project, starting at 1. IDs are project-scoped — task `3` in project A is unrelated to task `3` in project B. |
| **Status** | The lifecycle state of a task. One of: `new`, `in_progress`, `completed`. Transitions are free — any status can move to any other via `task move`. |
| **Priority** | An integer urgency level: `1` = low, `2` = medium, `3` = high. Controls display order within each status group (high first). |
| **Active Task** | The single task currently being worked on within a project. Each project tracks its own active task independently. Setting a task as active automatically moves its status to `in_progress`. Only one task per project can be active at a time. |
| **Focus** | The act of designating a task as the active task (`rtodo task set {id}`). Focuses attention and sets status to `in_progress`. |
| **Complete** | The act of moving the active task to `completed` and clearing the active marker (`rtodo task completed`). |

---

## Data Model

```
Workspace
└── state.json
    ├── active_project_id: int | null
    └── projects: [
          Project {
            id: int                  -- internal, auto-incremented
            name: string             -- unique within workspace
            created_at: datetime
            active_task_id: int | null
            tasks: [
              Task {
                id: int              -- sequential per project
                description: string
                status: new | in_progress | completed
                priority: 1 | 2 | 3  -- 1=low, 2=medium, 3=high
                created_at: datetime
              }
            ]
          }
        ]
```

---

## Storage

`rtodo` stores all data in `.rtodo/state.json` relative to your workspace root. The workspace is resolved by walking up from your current directory until a `.rtodo/` folder is found, mirroring how `git` discovers `.git/`.

---

## Commands

### Project Commands

```
rtodo new {name}
```
Create a new project. If `{name}` is omitted, defaults to the current directory name.

```
rtodo edit {project_name} {new_name}
```
Rename a project.

```
rtodo ls
```
List all projects. The active project is highlighted with `→`.

```
rtodo set {project_name}
```
Set the active project. Subsequent `task` commands will target this project.

```
rtodo delete {project_name}
```
Delete a project and all its tasks. Allowed even if the project is active.

---

### Task Commands

All task commands operate on the **active project**.

```
rtodo task ls [--status={new,in_progress,completed}]
```
List tasks as a tree, grouped by status (`in_progress` → `new` → `completed`), sorted by priority descending within each group. The active task is marked with `→`. Filter by `--status` to show only one group.

Example output:
```
→ [in_progress]
  3 !  Fix login bug          (active)
  1    Write unit tests

[new]
  2 !  Add dark mode

[completed]
  4    Setup CI
```
(`!` = priority 3, blank = priority 1 or 2)

```
rtodo task add {description} [--priority {1,2,3}]
```
Add a new task to the active project with status `new`. Priority defaults to `1` (low) if omitted.

```
rtodo task edit {id} {description}
```
Update the description of a task.

```
rtodo task set {id}
```
Focus on a task — marks it as the active task and moves its status to `in_progress`. Clears the active marker from the previously active task (that task remains `in_progress`).

```
rtodo task completed
```
Complete the active task — moves it to `completed` and clears the active marker.

```
rtodo task move {id} {new|in_progress|completed}
```
Freely move any task to any status. Does not affect the active marker. Useful for corrections and backtracking.

```
rtodo task delete {id}
```
Delete a task. Allowed even if the task is active.
