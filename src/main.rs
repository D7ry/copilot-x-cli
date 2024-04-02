use clap::{App, Arg};
mod chat;
mod codeblock_builder;
mod llm;
mod syntax;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use codeblock_builder::{CodeBlockBuilder, CodeBlockBuilderState};

use chat::Chat;
use std::io::{self, Write};
use termion::{clear, cursor, terminal_size};

use std::sync::{Arc, Mutex};
use std::thread;

fn print_separator() {
    let line_width = terminal_size().unwrap().0 as usize;
    println!("{}", "-".repeat(line_width));
    io::stdout().flush().unwrap();
}

fn print_prompt() {
    print!(">> ");
    io::stdout().flush().unwrap();
}

fn main_loop(conversation_starter: Option<String>) {
    let mut chat = Chat::new();
    let mut input: String = String::new();

    match conversation_starter {
        Some(msg) => {
            chat.ask(&msg);
        }
        None => {}
    }
    loop {
        input.clear();
        print_prompt();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        /* Handle special commands */
        {
            if input.starts_with("\\") && input.len() <= 3 {
                match input.as_str() {
                    "\\y" => {
                        println!("Yanking is wip!"); //TODO: add back yanking
                        continue;
                    }
                    "\\d" => {
                        // delete line
                        print!("{}", clear::CurrentLine);
                        io::stdout().flush().unwrap();

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
                        println!("\\cl - Clear screen");
                        print_separator();
                        continue;
                    }
                    "\\cl" => {
                        print!("\x1B[2J\x1B[1;1H");
                        io::stdout().flush().unwrap();
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
        let _response = chat.ask(&input);

        print_separator();
        std::io::stdout().flush().unwrap();
    }
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
                let mut copilot = Chat::new();
                copilot.ask(&msg);
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
