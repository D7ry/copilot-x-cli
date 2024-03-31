use crate::llm::{LLMRole, LLMMessage, LLM, CopilotChat};
use std::io::Write;



struct RuntimeState {
    line_buffer: String,
    line_buffer_unflushed_begin: usize,
}

pub struct Chat {
    messages: Vec<LLMMessage>,
    name: String,
    llm: dyn LLM,
    state: RuntimeState,
}

impl Chat {
    pub fn new() -> Chat {
        Chat {
            messages: Vec::new(),
            name: String::from("Chat"),
            llm: CopilotChat::new(),
            state : RuntimeState {
                line_buffer: String::new(),
                line_buffer_unflushed_begin: 0,
            }
        }
    }

    fn llm_response_callback(&self, response: &str) {
        
    }

    /**
     * Ask the assistant a question, and return the response
     */
    pub fn ask(&self, question: &str) -> String {

        
    }
}
