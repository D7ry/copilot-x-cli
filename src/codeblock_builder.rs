use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref MD_TYPE_TO_EXT: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("rust", "rs");
        m.insert("python", "py");
        m.insert("javascript", "js");
        m.insert("markdown", "md");
        m.insert("java", "java");
        m.insert("c", "c");
        m.insert("cpp", "cpp");
        m.insert("go", "go");
        m.insert("ruby", "rb");
        m.insert("swift", "swift");
        m.insert("kotlin", "kt");
        m.insert("scala", "scala");
        m.insert("php", "php");
        m.insert("html", "html");
        m.insert("css", "css");
        m.insert("typescript", "ts");
        m.insert("bash", "sh");
        m.insert("shell", "sh");
        m.insert("perl", "pl");
        m.insert("r", "r");
        m.insert("csharp", "cs");
        m.insert("fsharp", "fs");
        m.insert("haskell", "hs");
        m.insert("lua", "lua");
        m.insert("matlab", "m");
        m.insert("powershell", "ps1");
        m.insert("sql", "sql");
        m.insert("xml", "xml");
        m.insert("json", "json");
        m.insert("yaml", "yml");
        m.insert("objective-c", "m");
        m.insert("groovy", "groovy");
        m.insert("dart", "dart");
        m.insert("clojure", "clj");
        m.insert("elixir", "ex");
        m.insert("erlang", "erl");
        m.insert("fortran", "f");
        m.insert("prolog", "pl");
        m.insert("coffeescript", "coffee");
        m.insert("vbscript", "vbs");
        m.insert("pascal", "pas");
        m.insert("assembly", "asm");
        m.insert("racket", "rkt");
        m.insert("scheme", "scm");
        m.insert("ocaml", "ml");
        m.insert("julia", "jl");
        m.insert("elm", "elm");
        m.insert("smalltalk", "st");
        m.insert("vhdl", "vhd");
        m.insert("verilog", "v");
        m.insert("ada", "adb");
        m.insert("lisp", "lisp");
        m.insert("actionscript", "as");
        m.insert("apl", "apl");
        m.insert("awk", "awk");
        m.insert("cobol", "cbl");
        m.insert("d", "d");
        m.insert("f#", "fs");
        m.insert("j", "ijs");
        m.insert("labview", "vi");
        m.insert("logo", "logo");
        m.insert("ml", "ml");
        m.insert("nasm", "asm");
        m.insert("qml", "qml");
        m.insert("rpg", "rpg");
        m.insert("simula", "sim");
        m.insert("tcl", "tcl");
        m.insert("typescript", "js");
        m.insert("vb.net", "vb");
        m.insert("xquery", "xq");
        m.insert("zsh", "zsh");
        return m;
    };
}

pub enum CodeBlockBuilderState {
    None,
    EatingBackTicksBegin,
    BeginEatingCode,
    EatingCode,
    EndEatingCode,
}

pub struct CodeBlock {
    code: String,
    language_extension: String, // (c, rs, py, etc)
}

pub struct CodeBlockBuilder {
    backticks_count: u8,
    code_line_buf: String,
    code_block_type_buf: String,
    code_block_state: CodeBlockBuilderState,
    backticks_only_in_curr_line: bool,
    curr_code_block: CodeBlock, // currently building code block
}

impl CodeBlockBuilder {
    pub const fn new() -> CodeBlockBuilder {
        CodeBlockBuilder {
            backticks_count: 0,
            code_line_buf: String::new(),
            code_block_type_buf: String::new(),
            code_block_state: CodeBlockBuilderState::None,
            backticks_only_in_curr_line: false,
            curr_code_block: CodeBlock {
                code: String::new(),
                language_extension: String::new(),
            },
        }
    }

    /**
     * Reset the state of the code block builder
     */
    pub fn reset(&mut self) {
        self.backticks_count = 0;
        self.code_line_buf.clear();
        self.code_block_type_buf.clear();
        self.code_block_state = CodeBlockBuilderState::None;
        self.backticks_only_in_curr_line = false;
        self.curr_code_block.code.clear();
        self.curr_code_block.language_extension.clear();
    }

    /**
     * Ingests a character and tries to build a code block from it.
     * Returns:
     * (1. the current state of the code block builder,
     * (2. the code block if it was built, None otherwise. Each code block is built when the code block ends,
     * (3. the code line that just ended and the language of the code block if the code line ended , and the code block is still being built. This is useful for stream printing syntax-highlighted code lines.
     * Thank you Koushik Sen for all the compiler knowledge lmao
     */
    pub fn build_codeblock_from_char(
        &mut self,
        ch: char,
    ) -> (
        &CodeBlockBuilderState,
        Option<CodeBlock>,
        Option<(String, String)>,
    ) {
        // iterate over all chars, don't care if strs sent back are
        let mut code_block: Option<CodeBlock> = None;
        let mut code_line_and_language: Option<(String, String)> = None;
        if ch == '\n' {
            self.backticks_only_in_curr_line = true;
        }
        // println!("state: {:?}", MAIN_STATE.code_block_state);
        match self.code_block_state {
            // hopefully branch predictor carries performance
            CodeBlockBuilderState::None => {
                if ch == '\n' {
                    // hitting a new line, start DFA traversal
                    self.code_block_state = CodeBlockBuilderState::EatingBackTicksBegin;
                    self.code_block_type_buf.clear();
                }
            }
            CodeBlockBuilderState::EatingBackTicksBegin => {
                if self.backticks_count == 3 {
                    // take the chars between backticks and
                    // new line as the language of the code block

                    if ch == '\n' {
                        // we also ate this new line, so append this to the code block.
                        self.code_block_state = CodeBlockBuilderState::EatingCode;
                        self.backticks_count = 0;
                        self.curr_code_block.language_extension = MD_TYPE_TO_EXT
                            .get(self.code_block_type_buf.as_str())
                            .unwrap_or(&"txt")
                            .to_string();
                        self.curr_code_block.code.clear();
                        self.code_line_buf.push('\n');
                    } else {
                        self.code_block_type_buf.push(ch);
                    }
                } else {
                    // still counting backticks
                    match ch {
                        '`' => {
                            self.backticks_count += 1;
                        }
                        '\n' => {
                            self.backticks_count = 0;
                        }
                        _ => {
                            self.code_block_state = CodeBlockBuilderState::None;
                            self.backticks_count = 0;
                        }
                    }
                }
            }
            CodeBlockBuilderState::EatingCode => {
                self.code_line_buf.push(ch);
                match ch {
                    '\n' => {
                        if self.backticks_count >= 3 {
                            // end of code block
                            code_block = Some(CodeBlock {
                                code: self.curr_code_block.code.clone(),
                                language_extension: self.curr_code_block.language_extension.clone(),
                            });
                            self.code_block_state = CodeBlockBuilderState::None;
                            self.code_block_type_buf.clear();
                            self.backticks_count = 0;
                        }
                        code_line_and_language = Some((
                            self.code_line_buf.clone(),
                            self.curr_code_block.language_extension.clone(),
                        )); // send back the code line
                        self.curr_code_block.code.push_str(&self.code_line_buf);
                        self.code_line_buf.clear();
                    }
                    '`' => {
                        if self.backticks_only_in_curr_line {
                            self.backticks_count += 1;
                        }
                    }
                    _ => {
                        self.backticks_only_in_curr_line = false;
                    }
                }
            } // new line
        }
        return (&self.code_block_state, code_block, code_line_and_language);
    }
}
