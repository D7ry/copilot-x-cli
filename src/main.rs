use std::env;
mod llm;
use llm::{CopilotChat, LLM};
fn main_loop() {
    let mut llm = CopilotChat::new();
    let mut input: String = String::new();
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let _response = llm.ask(&input);
    }
}
fn main() {
    println!("Arguments: ");
    for arg in env::args() {
        println!("{}", arg)
    }
    main_loop();
}
