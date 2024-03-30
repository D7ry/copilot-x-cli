use std::env;
use clap::{Arg, App};
mod llm;
use llm::{CopilotChat, LLM};
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;
fn main_loop(conversation_starter: Option<String>) {
    let mut llm = CopilotChat::new();
    let mut input: String = String::new();

    match conversation_starter {
        Some(msg) => {
            let _response = llm.ask(&msg);
        },
        None => {
        }
    }
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        if input.contains("\\\\") { //input has double backlashes, replace them with clipboards content
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            match ctx.get_contents() {
                Ok(msg) => {
                    input = input.replace("\\\\", &msg);
                },
                Err(_) => {
                    println!("Error: Could not get clipboard contents when trying to replace \\\\ with clipboard contents");
                    return;
                }
            }
        }
        
        let _response = llm.ask(&input);
    }
}
fn main() {
    let mut conversation_starter : Option<String> = None;
    for arg in env::args() {
        println!("{}", arg)
    }
    
    let matches = App::new("Copilot Chat CLI")
        .arg(Arg::with_name("message")
            .short("m")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("single_query")
            .short("s")
            .takes_value(false)
            .required(false))
        .arg(Arg::with_name("use_clipboard")
            .short("c")
            .takes_value(false)
            .required(false))
        .arg(Arg::with_name("k")
            .short("k")
            .takes_value(true)
            .required(false))
        .get_matches();

    if matches.is_present("use_clipboard") {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        match ctx.get_contents() {
            Ok(msg) => {
                conversation_starter = Some(msg);
            },
            Err(_) => {
                println!("Error: Could not get clipboard contents");
                return;
            }
        }
    }

    match matches.value_of("message") {
        Some(msg) => {
            match &conversation_starter {
                Some(_) => {
                    conversation_starter.as_mut().unwrap().push_str(msg);
                },
                None => {
                    conversation_starter = Some(msg.to_string());
                }
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
                let _response = llm.ask(&msg);
            },
            None => {
                println!("Please provide a message to ask the model when doing single-time query");
                return;
            }
        }
    } else {
        main_loop(conversation_starter);
    }
    
}
