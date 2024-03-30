use std::env;
mod llm;
use llm::{CopilotChat, LLM};
fn main_loop() {
    let llm = CopilotChat::new();
    let mut input: String = String::new();
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        if input == "q" {
            break;
        }


        println!("you inputted: {}", input);
        let response = llm.ask(&input);
        println!("response: {}", response);
        
    }
}
fn main() {
    println!("Arguments: ");
    for arg in env::args() {
        println!("{}", arg)
    }
    
    println!("Hello, world!");

    main_loop();
}
