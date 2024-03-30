use clap::{App, Arg};
use std::env;
mod llm;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use llm::{CopilotChat, LLM};

use std::io::{self, Write};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use termion::{color, cursor};

use std::sync::{Arc, Mutex};

use std::thread;

#[derive(PartialEq, Debug)]
enum CodeBlockState {
    None,
    EatingBackTicksBegin,
    EatingBackTicksEnd,
    EatingCode,
}
struct MainState {
    backticks_count: u8,
    code_line_buf: String,
    code_block_type_buf: String,
    code_block_state: CodeBlockState,
    backticks_only_in_curr_line: bool,
}

fn lock_test() {
    let lock = Arc::new(Mutex::new(()));

    let lock_clone = Arc::clone(&lock);
    let handle1 = thread::spawn(move || {
        let _guard = lock_clone.lock().unwrap();
        // Critical section starts
        println!("Thread 1 is running");
        // Critical section ends
    });

    let lock_clone = Arc::clone(&lock);
    let handle2 = thread::spawn(move || {
        let _guard = lock_clone.lock().unwrap();
        // Critical section starts
        println!("Thread 2 is running");
        // Critical section ends
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

fn print_syntax_highlighted_code(code: &str, language: &str) {
    // Load the syntaxes and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Use the Python syntax
    let syntax = ps.find_syntax_by_extension("cpp").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let ranges: Vec<(Style, &str)> = h.highlight(code, &ps);
    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
    print!("{}", escaped);
    
    print!("\x1b[0m"); // reset color

    io::stdout().flush().unwrap();
}

static mut MAIN_STATE: MainState = MainState {
    backticks_count: 0,
    code_line_buf: String::new(), // one line of the code block
    code_block_type_buf: String::new(),
    code_block_state: CodeBlockState::None,
    backticks_only_in_curr_line: true,
};

fn print_separator() {
    println!("----------------------------------------");
    io::stdout().flush().unwrap();
}

fn llm_response_callback(response: &str) {
    unsafe {
        for ch in response.chars() { // iterate over all chars, don't care if strs sent back are
                                     // incomplete since we are processing at char level anyways
            if ch == '\n' {
                MAIN_STATE.backticks_only_in_curr_line = true;
            }
            if MAIN_STATE.code_block_state != CodeBlockState::EatingCode {
                print!("{}", ch);
            }
            // println!("state: {:?}", MAIN_STATE.code_block_state);
            match MAIN_STATE.code_block_state { // hopefully branch predictor carries performance
                CodeBlockState::None => {
                    if ch == '\n' { // hitting a new line, start DFA traversal
                        MAIN_STATE.code_block_state = CodeBlockState::EatingBackTicksBegin;
                        MAIN_STATE.code_block_type_buf.clear();
                    }
                }
                CodeBlockState::EatingBackTicksBegin => {
                    if MAIN_STATE.backticks_count == 3 { // take the chars between backticks and
                                                         // new line as the language of the code block
                        
                        if ch == '\n' {
                            // println!("begin code block: {}", MAIN_STATE.code_block_type_buf);
                            MAIN_STATE.code_block_state = CodeBlockState::EatingCode;
                            MAIN_STATE.backticks_count = 0;
                        } else {
                            MAIN_STATE.code_block_type_buf.push(ch);
                        }
                    } else { // still counting backticks
                        match ch {
                            '`' => {
                                MAIN_STATE.backticks_count += 1;
                            }
                            '\n'=> {
                                MAIN_STATE.backticks_count = 0;
                            }
                            _ => {
                                MAIN_STATE.code_block_state = CodeBlockState::None;
                                MAIN_STATE.backticks_count = 0;
                            }
                        }
                    }
                }
                CodeBlockState::EatingCode => {
                    MAIN_STATE.code_line_buf.push(ch);
                    match ch {
                        '\n' => {
                            if MAIN_STATE.backticks_count >= 3 { // exit code block
                                MAIN_STATE.code_block_state = CodeBlockState::None;
                                MAIN_STATE.code_block_type_buf.clear();
                                MAIN_STATE.code_line_buf.clear();
                                MAIN_STATE.backticks_count = 0;
                                // print the ending backticks that we swallowed
                                println!("```");
                            }
                            // print out formatted line of code
                            print_syntax_highlighted_code(
                                &MAIN_STATE.code_line_buf,
                                &MAIN_STATE.code_block_type_buf,
                            );
                            MAIN_STATE.code_line_buf.clear();
                        }
                        '`' => {
                            if MAIN_STATE.backticks_only_in_curr_line {
                                MAIN_STATE.backticks_count += 1;
                            }
                        }
                        _ => {
                            MAIN_STATE.backticks_only_in_curr_line = false;
                        }
                        
                    }
                    
                    
                }
                CodeBlockState::EatingBackTicksEnd => { // don't really need this state

                }
            }
            // new line
        }
    }
    // simply print the response
    io::stdout().flush().unwrap();
}

fn main_loop(conversation_starter: Option<String>) {
    let mut llm = CopilotChat::new();
    let mut input: String = String::new();

    match conversation_starter {
        Some(msg) => {
            let _response = llm.ask(&msg, llm_response_callback);
            print_separator();
        }
        None => {}
    }
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        /* Handle special commands */
        {
            if input.starts_with("\\") && input.len() == 2 {
                match input.as_str() {
                    "\\y" => {
                        let code_blocks = llm.get_code_blocks();
                        if code_blocks.len() == 0 {
                            println!("No code blocks to yank");
                            print_separator();
                            continue;
                        }
                        let last_code_block = code_blocks.into_iter().last().unwrap();
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        match ctx.set_contents(last_code_block.1.to_string()) {
                            Ok(_) => {
                                println!("Yanked {} code block to clipboard", last_code_block.0);
                                print_separator();
                            }
                            Err(_) => {
                                println!("Error: Could not yank code block to clipboard");
                            }
                        }
                        continue;
                    }
                    "\\p" => {
                        // do nothing, this is handled later
                    }
                    "\\h" => {
                        println!("Special commands:");
                        println!("\\q - Quit");
                        println!("\\h - Help");
                        println!("\\y - Yank last code block to clipboard");
                        print_separator();
                        continue;
                    }
                    _ => {
                        println!("Unknown comnad. Type \\h for help");
                        print_separator();
                        continue;
                    }
                }
            }
        }

        /* replace \p with clipboard contents */
        if input.contains("\\p") {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            match ctx.get_contents() {
                Ok(msg) => {
                    input = input.replace("\\p", &msg);
                }
                Err(_) => {
                    println!("Error: Could not get clipboard contents when trying to replace \\p with clipboard contents");
                    return;
                }
            }
        }

        print_separator();
        let _response = llm.ask(&input, llm_response_callback);
        println!();
        print_separator();
    }
}

fn test_syntax_highlighting() {
   let code = 
"
```python
#include <iostream>

int main() {
    std::cout << \"Hello, World!\" << std::endl;
    return 0;
}
```


    not code

    `` not code

    ` not code

not code



```python2
#include <iostream>\n#include <string>\n\nclass Person {\nprivate:\n    std::string name;\n    int age;\n\npublic:\n    Person(std::string name, int age) : name(name), age(age) {}\n\n    void greet() {\n        std::cout << \"Hello, my name is \\\"\" << name << \"\\\" and I am \" << age << \" years old.\\n\";\n    }\n\n    void haveBirthday() {\n        age++;\n        std::cout << \"It's my birthday! I am now \" << age << \" years old.\\n\";\n    }\n};\n\nint main() {\n    Person john(\"John\", 20);\n    john.greet();\n    john.haveBirthday();\n    john.greet();\n\n    return 0;\n}
```

not code


the above is beautiful!
    "; 
    llm_response_callback(code);
}

fn main() {
    // test_syntax_highlighting();
    // return;
    let mut conversation_starter: Option<String> = None;

    let matches = App::new("Copilot Chat CLI")
        .arg(
            Arg::with_name("message")
                .short("m")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("single_query")
                .short("s")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("use_clipboard")
                .short("c")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("k")
                .short("k")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    if matches.is_present("use_clipboard") {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        match ctx.get_contents() {
            Ok(msg) => {
                conversation_starter = Some(msg);
            }
            Err(_) => {
                println!("Error: Could not get clipboard contents");
                return;
            }
        }
    }

    match matches.value_of("message") {
        Some(msg) => match &conversation_starter {
            Some(_) => {
                conversation_starter.as_mut().unwrap().push_str(msg);
            }
            None => {
                conversation_starter = Some(msg.to_string());
            }
        },
        None => {
            // no thing, is pasted from clipboard the starter should be non-null
        }
    }

    if matches.is_present("single_query") {
        match conversation_starter {
            Some(msg) => {
                let mut llm = CopilotChat::new();
                let _response = llm.ask(&msg, llm_response_callback);
            }
            None => {
                println!("Please provide a message to ask the model when doing single-time query");
                return;
            }
        }
    } else {
        main_loop(conversation_starter);
    }
}
