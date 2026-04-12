# rtodo — Todo CLI in Rust

A small CLI to manage tasks organized by projects, local to your working directory.

---

## Installation

```sh
curl -LsSf https://raw.githubusercontent.com/yjmedina/rtodo/main/install.sh | sh
```

This installs `rtodo` to `~/.local/bin`. Override the install directory:

```sh
RTODO_INSTALL_DIR=~/.bin curl -LsSf https://raw.githubusercontent.com/yjmedina/rtodo/main/install.sh | sh
```

**Requirements:** `curl`, `tar`, `sha256sum` or `shasum`

---

## Usage

`rtodo` stores state in a `.rtodo/` folder at your workspace root — discovered by walking up from your current directory, similar to how `git` finds `.git/`.

Run `rtodo --help` or `rtodo <command> --help` for details on any command.

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
Delete a project and all its tasks.

---

### Task Commands

All task commands operate on the **active project**.

```
rtodo task ls [--status={new,in_progress,completed}]
```
List tasks grouped by status (`in_progress` → `new` → `completed`), sorted by priority. The active task is marked with `→`.

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
(`!` = high priority)

```
rtodo task add {description} [--priority {1,2,3}]
```
Add a new task. Priority defaults to `1` (low). `3` = high.

```
rtodo task edit {id} {description}
```
Update a task's description.

```
rtodo task set {id}
```
Focus on a task — marks it active and moves status to `in_progress`.

```
rtodo task completed
```
Complete the active task — moves it to `completed` and clears the active marker.

```
rtodo task move {id} {new|in_progress|completed}
```
Move any task to any status. Useful for corrections and backtracking.

```
rtodo task delete {id}
```
Delete a task.
