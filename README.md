# NAME

**tafsk** - Organize tasks like a file system.

# SYNOPSIS

**tafsk** [*OPTIONS*] [*SUBCOMMAND*] [*ARGS*]

# DESCRIPTION

**tafsk** is a command-line utility for managing tasks as plaintext using a
filesystem-based approach: basically, every task is a directory. On creation,
tasks are given a **HU**man **ID**entifier derived from the current datetime
plus a hash of its contents. Extra information can be attached to a task by
adding files to its directory. You can even keep it under version control.

Derive from this what value you can find.

# COMMANDS

## add

Create a new task.

**USAGE**

**tafsk add** [**-p**|**--priority** *N*] [**-e**|**--edit**] [*+TAG*...] *TITLE*

**OPTIONS**

  - **-p**, **--priority** *N*
    Set the priority level for the task (0-255). The default priority is 10.

  - **-e**, **--edit**
    Open the newly created task on the default editor.

  - *TITLE*
    The short description of the task.

  - *+TAG*
    One or more tags to categorize the task. Tags must be prefixed with a `+`.

**EXAMPLE**

```bash
tafsk add -p 100 "Fix critical bug" +work
tafsk add -p 60 +cat Send cat pictures to smelly
```

## done

Mark one or more tasks as closed.

**USAGE**

**tafsk done** *ID*...

**ARGS**

  - *ID*
    The numeric ID of the task(s) to close. IDs correspond to the numbers
    displayed in the `list` command output.

**EXAMPLE**

```bash
tafsk done 1 3
```

## init

Initialize the task store configuration.

**USAGE**

**tafsk init** [*TIMEZONE*]

**ARGS**

  - *TIMEZONE*
    The timezone offset to be used for the store (e.g., `+09:30`,
    `-05:00`). Defaults to `+00:00` (UTC) if not specified.

**EXAMPLE**

```bash
tafsk init -03:00
```

## list

List all tasks. This is the default action when no subcommand is provided.

Tasks are listed in the following format:

      {PATH}:7:{ID}: [PRIORITY: {PRIORITY}] [TAGS: {TAGS}] {TITLE}

This format was chosen because it plays nicely with neovim's quickfix list.

**USAGE**

**tafsk list** [**-c**|**--closed**]

**OPTIONS**

  - **-c**, **--closed**
    Include closed tasks in the output list. By default, only open tasks are
    shown.

**EXAMPLE**

```bash
tafsk list --closed
```

# GLOBAL OPTIONS

  - **-h**, **--help**
    Print help information.

  - **-V**, **--version**
    Print version information.

# ENVIRONMENT

  - **EDITOR**
    Specifies the editor to use when the **--edit** flag is passed to the **add**
    command. If this environment variable is not set, **tafsk** defaults to `vi`.

# ACKNOWLEDGEMENTS

- **Tsoding** - For the idea. <https://www.youtube.com/watch?v=QH6KOEVnSZA>

# BUGS

Report issues at: <https://github.com/yilisharcs/tafsk/issues>

# LICENSE

Copyright (C) 2025 yilisharcs <yilisharcs@gmail.com>

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>.
