# 0-shell 🐚

A minimalist Unix-like shell written in **Rust**, designed to reproduce essential Unix shell behavior without relying on external binaries such as `bash`, `sh`, or system utilities.

The project focuses on understanding how shells work internally, including command parsing, file-system operations, process management, input/output handling, and Unix system APIs.

---

## 📖 Overview

**0-shell** is a lightweight command-line shell implemented from scratch in Rust.

The shell provides a set of essential Unix commands while relying on Rust's standard library and Unix APIs instead of spawning existing system commands.

The project is inspired by lightweight Unix environments such as [BusyBox](https://busybox.net/) and aims to provide hands-on experience with system-level programming.

---

## 🎯 Learning Objectives

Through this project, you will learn how to:

* Work with files and directories.
* Read and process user input.
* Build an interactive shell loop.
* Parse command-line arguments.
* Implement Unix-like commands.
* Handle errors safely.
* Work with Unix file metadata and permissions.
* Understand process and system-level APIs.
* Use Rust abstractions for system programming.

---

## ⚙️ Requirements

The shell must:

* Display a `$ ` prompt.
* Wait for user input.
* Parse the entered command.
* Execute the requested operation.
* Wait until the command finishes.
* Return to the prompt.
* Handle `Ctrl+D` (EOF) gracefully.
* Display an error when an unknown command is entered.

For an unknown command, the shell prints:

```text
Command '<name>' not found
```

---

## 🛠️ Implemented Commands

The following commands are implemented from scratch:

| Command | Description                           |
| ------- | ------------------------------------- |
| `echo`  | Display text                          |
| `cd`    | Change the current directory          |
| `ls`    | List files and directories            |
| `pwd`   | Display the current working directory |
| `cat`   | Display file contents                 |
| `cp`    | Copy files                            |
| `rm`    | Remove files and directories          |
| `mv`    | Move or rename files                  |
| `mkdir` | Create directories                    |
| `exit`  | Exit the shell                        |

### `ls` options

The `ls` command supports:

```text
-l    Long listing format
-a    Show hidden files
-F    Classify file types
```

Examples:

```bash
ls
ls -l
ls -a
ls -l -a
ls -la
ls -lF
```

---

## 📂 Project Structure

A possible project structure is:

```text
0-shell/
├── Cargo.toml
├── Cargo.lock
├── README.md
└── src/
    ├── main.rs
    └── commands/
        ├── mod.rs
        ├── echo.rs
        ├── cd.rs
        ├── ls.rs
        ├── pwd.rs
        ├── cat.rs
        ├── cp.rs
        ├── rm.rs
        ├── mv.rs
        ├── mkdir.rs
        └── exit.rs
```

Each command is implemented independently, making the project easier to maintain and extend.

---

## 🚀 Installation

Make sure Rust and Cargo are installed:

```bash
rustc --version
cargo --version
```

Clone the repository:

```bash
git clone <repository-url>
cd 0-shell
```

Build the project:

```bash
cargo build
```

For a release build:

```bash
cargo build --release
```

---

## ▶️ Running the Shell

Run the project with Cargo:

```bash
cargo run
```

Or execute the compiled binary:

```bash
./target/debug/0-shell
```

You should see the shell prompt:

```text
$ 
```

---

## 💻 Example Usage

```text
$ pwd
/home/student

$ mkdir dev

$ cd dev

$ pwd
/home/student/dev

$ echo Hello There
Hello There

$ ls
file.txt
src

$ ls -la
total ...
.
..
file.txt
src

$ cat file.txt
Hello from 0-shell!

$ something
Command 'something' not found

$ cd ..

$ rm -r dev

$ exit
```

---

## 🔍 `Ctrl+D`

Pressing:

```text
Ctrl+D
```

sends **EOF (End Of File)** to the shell.

The shell should detect this condition and exit gracefully instead of crashing or repeatedly displaying the prompt.

Example:

```text
$ 
^D
```

The program terminates.

---

## 🧩 Command Parsing

The shell accepts basic command syntax.

For example:

```text
echo Hello World
```

The command is separated into:

```text
echo
Hello
World
```

The first element represents the command, while the remaining elements are its arguments.

The project intentionally does **not** require advanced shell parsing.

The following features are not part of the core requirements:

```text
|
>
<
*
```

Therefore, commands such as pipes, redirections, and globbing do not need to be supported.

---

## 📁 File-System Operations

Several commands interact directly with the filesystem.

For example:

### `pwd`

Retrieves the current working directory.

```text
$ pwd
/home/student/project
```

### `mkdir`

Creates a directory:

```text
$ mkdir test
```

### `cd`

Changes the current directory:

```text
$ cd test
```

### `cp`

Copies a file:

```text
$ cp file.txt backup.txt
```

### `mv`

Moves or renames a file:

```text
$ mv backup.txt old.txt
```

### `rm`

Removes a file:

```text
$ rm old.txt
```

Recursive removal:

```text
$ rm -r test
```

---

## 🔐 Unix File Metadata

The `ls -l` implementation works with Unix filesystem metadata.

It can display information such as:

```text
-rw-r--r-- 1 user user 1234 Aug 10 20:15 file.txt
```

The implementation handles:

* File type
* Permissions
* Number of links
* User ID / username
* Group ID / group name
* File size
* Modification time
* Symbolic links

The `-F` option can additionally classify entries:

```text
directory/
executable*
symlink@
fifo|
socket=
```

---

## ❌ Error Handling

The shell should handle invalid operations without crashing.

For example:

```text
$ cd unknown
cd: unknown: No such file or directory
```

Or:

```text
$ cat missing.txt
cat: missing.txt: No such file or directory
```

Unknown commands produce:

```text
$ hello
Command 'hello' not found
```

The shell should remain active after an error:

```text
$ hello
Command 'hello' not found

$ pwd
/home/student
```

---

## 🚫 Constraints

The project does not rely on external shell commands.

For example, the implementation should **not** execute:

```text
/bin/ls
/bin/cp
/bin/rm
/bin/mkdir
```

and should not delegate commands to:

```text
bash
sh
```

Instead, commands are implemented using Rust's filesystem and Unix APIs.

---

## 🌟 Bonus Features

The following features can be added as extensions:

* `Ctrl+C` / SIGINT handling
* Command auto-completion
* Command history
* Current-directory prompt
* Colored output
* Command chaining with `;`
* Pipes with `|`
* Input/output redirection
* Environment variables such as `$HOME` and `$PATH`
* A custom `help` command

For example, a more advanced prompt could look like:

```text
~/projects/0-shell $
```

---

## 🧪 Testing

Build the project:

```bash
cargo build
```

Run the tests:

```bash
cargo test
```

Run the shell:

```bash
cargo run
```

Then test each command manually:

```text
echo
cd
pwd
ls
cat
cp
rm
mv
mkdir
exit
```

Also test invalid inputs and missing files/directories.


---

## 📌 Evaluation Criteria

The project is evaluated mainly on:

### Functionality

Commands should behave correctly and follow standard Unix conventions.

### Stability

The shell should:

* Handle invalid commands.
* Handle missing files.
* Handle invalid arguments.
* Handle filesystem errors.
* Avoid crashing on user mistakes.
* Handle `Ctrl+D` correctly.

### Code Quality

The implementation should follow good coding practices, remain readable, and separate responsibilities between different commands.

---

## 👨‍💻 Project Goal

The main goal of **0-shell** is not simply to reproduce a terminal.

It is to understand what happens behind common Unix commands and how a shell interacts with the operating system.

By implementing commands directly in Rust, the project provides practical experience with:

```text
User Input
    ↓
Command Parsing
    ↓
Command Selection
    ↓
Rust / Unix APIs
    ↓
Filesystem / Process Operations
    ↓
Output
    ↓
Shell Prompt
```

---

## 📄 License

This project was created for educational purposes as part of a system-programming exercise.
