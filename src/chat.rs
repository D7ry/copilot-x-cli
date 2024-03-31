use crate::llm::LLMRole;
use std::io::Write;

struct Message {
    owner: LLMRole,
    content: String,
}


struct RuntimeState {
    
}

pub struct Chat {
    messages: Vec<Message>,
    name: String,
    state: RuntimeState,
}

impl Chat {
    pub fn new() -> Chat {
        Chat {
            messages: Vec::new(),
            name: String::from("Chat"),
        }
    }

    pub fn ask(&self, question: &str, out: ) -> String {
    }
}
