use std::io::{self, Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use std::collections::HashMap;

pub fn print_syntax_highlighted_code_block(code_block: &str, language: &str) {
    let blocks = code_block.split("\n");

    for block in blocks {
        print_syntax_highlighted_code_line(block, language, None);
    }
}

/**
 * Print syntax highlighted code to the terminal
 *
 * @param code: &str - The code to print. The code should be a single line
 * @param language: &str - The language's extension(example: "rs" for Rust)
 */
pub fn print_syntax_highlighted_code_line(code: &str, language: &str, begin: Option<usize>) {
    let s = get_syntax_highlighted_code_line(code, language, begin);
    print!("{}", s);
    io::stdout().flush().unwrap();
}


pub fn get_syntax_highlighted_code_line(code: &str, language: &str, begin: Option<usize>) -> String {
    // Load the syntaxes and themes
    // println!("printing |{} : {} |", language, code);
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    // println!("=========================================");

    // println!("beign: {:?}", begin);
    // println!("code line: {}", code.len());

    // use C syntax if no syntax is found

    let syntax = ps.find_syntax_by_extension(language).unwrap();

    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let ranges: Vec<(Style, &str)> = h.highlight(code, &ps);
    // println!("ranges: {:?}", ranges);

    let mut ranges_post = Vec::new();

    let mut i: i16 = 0;

    // println!("begin: {:?}", begin);

    // TODO: this can be done more efficiently?
    if let Some(begin) = begin {
        let begin = begin as i16;
        let mut start_appending = false;
        for (style, text) in &ranges {
            // println!("tex len: {}", text.len());
            let text_len = text.len() as i16;
            if i + text_len >= begin  {
                // println!("i: {}, text_len: {}, begin: {}", i, text_len, begin);
                let diff = i + text_len - begin;
                let text_size = std::cmp::min(diff, text_len);
                let slice_index = text_len - text_size;
                // println!("diff: {}", diff);
                // slice the cutoff text
                // println!("slice_index: {}", slice_index);
                ranges_post.push((style.clone(), &text[slice_index as usize..]));
                start_appending = true;
            }
            i += text_len;
        }
        if !start_appending {
            ranges_post = ranges.clone();
        }
    }
    // println!("ranges_post: {:?}", ranges_post);
    let mut escaped = as_24_bit_terminal_escaped(&ranges_post[..], false);

    escaped.push_str("\x1b[0m");
    return escaped;
}
