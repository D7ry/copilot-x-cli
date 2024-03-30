
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use std::io::{self, Write};


use std::collections::HashMap;


/**
 * Print syntax highlighted code to the terminal
 *
 * @param code: &str - The code to print. The code should be a single line
 * @param language: &str - The language's extension(example: "rs" for Rust)
 */
pub fn print_syntax_highlighted_code_line(code: &str, language: &str) {
    // Load the syntaxes and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // use C syntax if no syntax is found

    let syntax = ps.find_syntax_by_extension(language).unwrap();

    
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let ranges: Vec<(Style, &str)> = h.highlight(code, &ps);
    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
    print!("{}", escaped);

    print!("\x1b[0m"); // reset color
    io::stdout().flush().unwrap();
}
