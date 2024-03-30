use std::env;
use clap::{Arg, App};
mod llm;
use llm::{CopilotChat, LLM};
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

        let _response = llm.ask(&input);
    }
}
fn main() {
    let conversation_starter : Option<String>;
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
        .arg(Arg::with_name("j")
            .short("j")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("k")
            .short("k")
            .takes_value(true)
            .required(false))
        .get_matches();


    match matches.value_of("message") {
        Some(msg) => {
            conversation_starter = Some(msg.to_string());
        },
        None => {
            conversation_starter = None;
        }
    }


    if matches.is_present("single_query") {
        let query : String;
        match matches.value_of("message") {
            Some(msg) => {
                query = msg.to_string();
            },
            None => {
                println!("Please provide a query to ask");
                return;
            }
        }
        
        let mut llm = CopilotChat::new();
        let _response = llm.ask(&query);
        return;
    } else {
        main_loop(conversation_starter);
    }
    
}
