use clap::{App, Arg};
mod codeblock_builder;
mod llm;
mod syntax;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use codeblock_builder::{CodeBlockBuilder, CodeBlockBuilderState};
use llm::{CopilotChat, LLM};

use std::io::{self, Write};
use termion::{clear, cursor, terminal_size};

static mut CODEBLOCK_BUILDER: CodeBlockBuilder = CodeBlockBuilder::new();

fn print_separator() {
    println!("----------------------------------------");
    io::stdout().flush().unwrap();
}

static mut LINEBUFFER: String = String::new();
static mut LINEBUFFER_UNFLUSHED_BEGIN: usize = 0;

fn llm_response_callback(response: &str) {
    // println!("Response: {}", response);
    // println!("callback!");
    for ch in response.chars() {
        let print_char: bool;
        unsafe {
            let res = CODEBLOCK_BUILDER.build_codeblock_from_char(ch);
            match res.0 {
                CodeBlockBuilderState::EatingCode => {
                    print_char = false;
                }
                CodeBlockBuilderState::BeginEatingCode => {
                    print!("{}", cursor::Left(LINEBUFFER.len() as u16));
                    print!("{}", clear::UntilNewline);
                    std::io::stdout().flush().unwrap();
                    syntax::print_syntax_highlighted_code_line(
                        LINEBUFFER.as_str(),
                        "md",
                        Some(LINEBUFFER_UNFLUSHED_BEGIN),
                    );
                    LINEBUFFER.clear();
                    LINEBUFFER_UNFLUSHED_BEGIN = 0;
                    print_char = false;
                }
                _ => {
                    print_char = true;
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
        }

        if print_char {
            unsafe {
                let size = terminal_size();
                let w = size.unwrap().0;
                {
                    LINEBUFFER.push(ch);
                    // print the line buffer and set index to the end of the line buffer
                    if (LINEBUFFER.len() - LINEBUFFER_UNFLUSHED_BEGIN) >= w as usize {
                        print!("{}", cursor::Left(LINEBUFFER.len() as u16));
                        print!("{}", clear::UntilNewline);
                        std::io::stdout().flush().unwrap();
                        syntax::print_syntax_highlighted_code_line(
                            LINEBUFFER.as_str(),
                            "md",
                            Some(LINEBUFFER_UNFLUSHED_BEGIN),
                        );
                        LINEBUFFER_UNFLUSHED_BEGIN = LINEBUFFER.len();
                    } else if ch == '\n' {
                        print!(
                            "{}",
                            cursor::Left((LINEBUFFER.len() - LINEBUFFER_UNFLUSHED_BEGIN) as u16)
                        );
                        print!("{}", clear::UntilNewline);
                        std::io::stdout().flush().unwrap();
                        syntax::print_syntax_highlighted_code_line(
                            LINEBUFFER.as_str(),
                            "md",
                            Some(LINEBUFFER_UNFLUSHED_BEGIN),
                        );
                        LINEBUFFER.clear();
                        LINEBUFFER_UNFLUSHED_BEGIN = 0;
                    } else {
                        print!("{}", ch);
                    }
                }
            }
        }
    }

    io::stdout().flush().unwrap();
    // simply print the response
}

fn main_loop(conversation_starter: Option<String>) {
    let mut llm = CopilotChat::new();
    let mut input: String = String::new();

    match conversation_starter {
        Some(msg) => {
            unsafe {
                CODEBLOCK_BUILDER.reset();
            }
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
                        println!("Code blocks: {:?}", code_blocks);
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
        unsafe {
            CODEBLOCK_BUILDER.reset();
        }
        let _response = llm.ask(&input, llm_response_callback);

        print_separator();
        std::io::stdout().flush().unwrap();

        unsafe {
            assert!(LINEBUFFER.is_empty());
        }
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
