
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use std::io::{self, Write};


use std::collections::HashMap;
use lazy_static::lazy_static;


lazy_static! {
    static ref SYNTAX_MAP: HashMap<&'static str, &'static str> = {
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
        m.insert("typescript", "ts");
        m.insert("vb.net", "vb");
        m.insert("xquery", "xq");
        m.insert("zsh", "zsh");
        return m;
    };
}

/**
 * Print syntax highlighted code to the terminal
 *
 * @param code: &str - The code to print. The code should be a single line
 * @param language: &str - The markdown syntax indicator of the language.
 */
pub fn print_syntax_highlighted_code_line(code: &str, language: &str) {
    // Load the syntaxes and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // use C syntax if no syntax is found
    let syntax;

    match SYNTAX_MAP.get(language) {
        Some(s) => 
            match ps.find_syntax_by_extension(s) {
                Some(s) => {
                    syntax = s;
                }
                None => syntax = ps.find_syntax_by_extension("c").unwrap(),
            },
        None => syntax = ps.find_syntax_by_extension("c").unwrap(),
    }

    
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let ranges: Vec<(Style, &str)> = h.highlight(code, &ps);
    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
    print!("{}", escaped);

    print!("\x1b[0m"); // reset color
    io::stdout().flush().unwrap();
}
