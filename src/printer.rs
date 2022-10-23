use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub fn highlight(name: String, code: String) -> Result<String, String> {
    let synset = SyntaxSet::load_defaults_newlines();
    let themeset = ThemeSet::load_defaults();

    let cursyntax = match synset.find_syntax_by_name(&name) {
        Some(s) => s,
        _ => return Err(String::from("failed to load syntax by name for ") + &name)
    };
    let mut hl = HighlightLines::new(cursyntax, &themeset.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(&code) {
        let lines = hl.highlight_line(line, &synset).unwrap();
        let output = as_24_bit_terminal_escaped(&lines[..], true);
        return Ok(output);
    }
    return Err(String::from("??"));
}