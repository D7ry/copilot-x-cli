use crate::codeblock_builder::{CodeBlockBuilder, CodeBlockBuilderState};
use crate::llm::{CopilotChat, LLMMessage, LLMRole, LLM};
use crate::syntax;
use std::io::{self, Write};
use termion::{clear, cursor, terminal_size};

use std::mem;
struct LLMResponsePrinter {
    line_buffer: String,
    word_buffer: String,
    line_buffer_unflushed_begin: usize,
    codeblock_builder: CodeBlockBuilder,
    line_width: usize,
}

impl LLMResponsePrinter {
    fn llm_response_callback(&mut self, response: &str) {
        for ch in response.chars() {
            let char_is_code_block: bool;
            let res = self.codeblock_builder.build_codeblock_from_char(ch);
            match res.0 {
                CodeBlockBuilderState::EatingCode => {
                    char_is_code_block = false;
                }
                CodeBlockBuilderState::BeginEatingCode => {
                    let line = syntax::get_syntax_highlighted_code_line(
                        self.line_buffer.as_str(),
                        "md",
                        Some(self.line_buffer_unflushed_begin),
                    );
                    print!("{}", cursor::Left(self.line_buffer.len() as u16));
                    print!("{}", clear::UntilNewline);
                    std::io::stdout().flush().unwrap();
                    print!("{}", line);
                    self.line_buffer.clear();
                    self.line_buffer_unflushed_begin = 0;
                    char_is_code_block = false;
                }
                _ => {
                    char_is_code_block = true;
                }
            }
            match res.1 {
                Some(code_block) => {
                    //TODO: create app object to manage states
                }
                None => {}
            }
            match res.2 {
                Some(code_line_and_language) => {
                    syntax::print_syntax_highlighted_code_line(
                        code_line_and_language.0.as_str(),
                        code_line_and_language.1.as_str(),
                        Some(0),
                    );
                }
                None => {}
            }

            if char_is_code_block {
                // not code block. pirnt as markdown
                fn print_line_buffer (line_buffer: &String, begin: usize) {
                    let line = syntax::get_syntax_highlighted_code_line(
                        line_buffer.as_str(),
                        "md",
                        Some(begin),
                    );
                    print!("{}", cursor::Left(999));
                    print!("{}", clear::UntilNewline);
                    print!("{}", line);
                    std::io::stdout().flush().unwrap();
                };

                fn push_word_buffer (word_buffer_ref: &mut String, line_buffer_ref: &mut String) {
                    line_buffer_ref.push_str(word_buffer_ref);
                    word_buffer_ref.clear();
                };

                let size = terminal_size();
                let w = size.unwrap().0;
                let line_width_limit = std::cmp::min(w as usize, self.line_width);
                let curr_line_size: usize =
                    self.line_buffer.len() - self.line_buffer_unflushed_begin;
                self.word_buffer.push(ch);

                let curr_word_size: usize = self.word_buffer.len();
                let line_too_long_with_new_word =
                    curr_line_size + curr_word_size >= line_width_limit as usize;

                let should_push_word = ch == ' ' || ch == '\n';

                // pushing the word does not make the line too long
                if should_push_word && !line_too_long_with_new_word {
                    push_word_buffer(&mut self.word_buffer, &mut self.line_buffer);
                }
                // print the line buffer and set index to the end of the line buffer
                if line_too_long_with_new_word {
                    // we don't clear the buffer here here because we need the line buffer as context
                    // infor for syntax highlighting
                    print_line_buffer(&self.line_buffer, self.line_buffer_unflushed_begin);
                    println!(); // manually insert a new line
                    self.line_buffer_unflushed_begin = self.line_buffer.len();
                } else if ch == '\n' {
                    print_line_buffer(&self.line_buffer, self.line_buffer_unflushed_begin);
                    self.line_buffer.clear();
                    self.line_buffer_unflushed_begin = 0;
                } else {
                    print!("{}", ch); // simply prints out the char, which will get erased once
                                      // the syntax-highlighted line is printed
                }
                // push the word after the line is printed
                if should_push_word && line_too_long_with_new_word {
                    push_word_buffer(&mut self.word_buffer, &mut self.line_buffer);
                }
            }
        }

        io::stdout().flush().unwrap();
    }
}

static mut RESPONSE_HANDLER: LLMResponsePrinter = LLMResponsePrinter {
    word_buffer: String::new(),
    line_buffer: String::new(),
    line_buffer_unflushed_begin: 0,
    codeblock_builder: CodeBlockBuilder::new(),
    line_width: 80,
};

pub struct Chat {
    chat_history: Vec<LLMMessage>,
    name: String,
    copilot: CopilotChat,
}

impl Chat {
    pub fn new() -> Chat {
        Chat {
            chat_history: Vec::new(),
            name: String::from("Chat"),
            copilot: CopilotChat::new(),
        }
    }

    /**
     * Ask the assistant a question, and return the response
     */
    pub fn ask(&mut self, question: &str) -> String {
        self.chat_history.push(LLMMessage {
            owner: LLMRole::User,
            content: question.to_string(),
        });

        let response = self.copilot.query(&self.chat_history, |response| {
            {
                // fuck it, let's get this to compile first
                unsafe {
                    RESPONSE_HANDLER.llm_response_callback(response);
                }
            }
        });

        let ai_response;
        match response {
            Ok(msg) => {
                self.chat_history.push(LLMMessage {
                    owner: LLMRole::Assistant,
                    content: msg.clone(),
                });
                ai_response = msg;
            }
            Err(e) => {
                match e.status() {
                    Some(status_code) => match status_code.as_u16() {
                        400 => {
                            println!(
                                "Request rejected by API with code 400. Consider asking again."
                            )
                        }
                        _ => {
                            println!("Error: {}", status_code);
                        }
                    },
                    None => {
                        println!("Unknown error when executing copilot query.");
                    }
                }
                ai_response = "".to_string();
            }
        }

        return ai_response;
    }
}
