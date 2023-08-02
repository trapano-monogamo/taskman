# TASKMAN - Task Manager system
---

## TODOs

- [x] move the execution of the commands to a different function, and keep in the process_input function
	just the processing, which should send to the execute_command function the processed commands as an enum like this:

```rust
	enum Command {
		Help,
		List(SortBy),
		Add(Priority, String),
		Remove(u32),
		Priority(u32, Priority),
		Status(u32, Status),
		Exit,
		None,
	}
```

	- make process_input return a Command, which is then processed in result pattern matching

- [ ] refactor the input processing to make commands easier.
	Sytanx example: $ cmd <arg> [opt]
	
		$ add <title> [description] [priority]
		$ remove <id>
		$ status <id> <new_status>
		...
	
	- extract first word by splitting with ' '
	- treat quotes as single argument by joining with ' ' all elements between quotes:

		$ add "write readme" "include todos, commands explanation and code structure" medium
			--> [add, "write, readme", "include, todos,, commands, ..., structure", medium]
					  ^-------------^  ^-----------------------------------------^
			--> [add, "...", "...", medium]

- [ ] code saving to a file at Command::Exit
	- write tasklist to $HOME/.taskman

- [ ] design tui elements:
	- a Box should be a rectangle that contains formatted text (with wrapping and trimming).
		- maybe a block has a fixed size but the border is displayed according to the number of lines of the text
	- a Panel should contain positioned and sized Boxs.
	  Only one Panel will be displayed, it's just easier to store blocks in a dedicated struct
	- the Prompt should display custom text and a custom prompt symbol (e.g. "> ")
	  and is where the cursor will be positioned while waiting for user input.
	  The prompt is always on the bottom, under the Panel.
	

```
	Panel ---+
	         |
             v
	+---------------------------------------------------------------------------------------+
	| +---ToDo-pp----------------+ +---Doing-------------------+ +---Done-----------------+ |
	| |  8. [** ] code           | |  5. [*  ] write the todos | |  3. [***] build taskma-| |
	| |  2. [*  ] design the tui | | 10. [** ] ...             | |  4. [** ] ...          | |
	| |           elements       | +---------------------------+ |                     <--------- formatted task list
	| | 11. [***] do something   |                               |  9. [***] .....        | |
	| +--------------------------+                               |                        | |
	|                                                            |  7. [*  ] kjlfevwafe   | |
	|                                                            |  1. [** ] fewa rearffe-| |
	|                                                            +------------------------+ |
	|                                                                        ^              |
	|                                                                        |              |
	|                                                                        +------------------- Box
	|                                                                                       |
	|                                                                                       |
	|                                                 +--------------------------------+--------- error logging and command history
	|                                                 |                                |    |
	| +---Errors--------------------------------------|--+ +---Command History---------|--+ |
	| | [Error] 'asdf' is not a valid command...      v  | | > asdf:2,doing            v  | |
	| | [Error] could not find task with id '1'...       | | > status:1,doing             | |
	| |                                                  | | > priority:11,high           |<----- Box
	| +--------------------------------------------------+ +------------------------------+ |
	+---------------------------------------------------------------------------------------+     
	| <command>:<arg1,arg2,...>                                                             |
	| help, list, add, remove, priority, status, exit                                       |<--- Prompt is for user input and contains the cursor
	| $ _                                                                                   |
	+---------------------------------------------------------------------------------------+
```

```rust
	struct TUI {
		// ...
		panel: Panel,
		prompt: Prompt,
		err_msgs: Vec<String>,
		cmd_hist: Vec<String>
    }

	struct Box {
		position: [i32;2],
		size: [i32;2],
		name: String,
		text: String,
    }
	
	struct Panel {
        blocks: Vec<Box>,
    }
	
	struct Prompt {
		text: String,
		symbol: String,
    }
```
