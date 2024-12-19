# Shell Implementation in Rust

This repository contains my implementation of a shell as part of the [CodeCrafters.io Shell Challenge](https://codecrafters.io). Built entirely in Rust, this shell replicates basic Unix shell functionalities.

---

## Current Features

### Built-in Commands
- **`cd`**: Change the current working directory.
- **`pwd`**: Display the current working directory.
- **`echo`**: Print arguments to the standard output.
- **`type`**: Display whether a command is a built-in or an external executable.
- **`exit`**: Terminate the shell.

### External Command Execution
- Seamlessly runs external binaries available in the system PATH.
- Displays standard output and standard error from commands.

### Robust Path Resolution
- Handles relative paths, absolute paths, and home directory shortcuts (`~`).
- Ensures proper context management for path changes and directory traversal.

### Context Management
- Maintains state for executables and the current working directory.
- Dynamically updates the list of available executables from the system PATH.

### Command Parsing
- Supports handling single and double-quoted strings.
- Escapes special characters in commands where appropriate.

---

## Next Steps (TODOs)

### Enhanced Shell Features
- [ ] Add support for I/O redirection (e.g., `>`, `<`, `>>`).
- [ ] Implement piping (e.g., `command1 | command2`).
- [ ] Add tab-completion for commands and file paths.
- [ ] Support environment variable management (e.g., `export`, `unset`).

### Usability Improvements
- [ ] Add a history feature to navigate previous commands.

### Testing and Optimization
- [ ] Add unit and integration tests for core functionalities.
- [ ] Optimize performance for large or complex commands.

---

## Getting Started

1. Clone this repository:
   ```bash
   git clone https://github.com/omagdy7/tiny-shell
   cd tiny-shell
   ```

2. Build and run the project:
   ```bash
   cargo run --release
   ```

---

## Acknowledgments

Special thanks to [CodeCrafters.io](https://codecrafters.io) for designing this challenge and providing an engaging platform for learning system-level programming.


## Resources
- https://www.gnu.org/software/bash/manual/bash.html
