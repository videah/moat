use syntect::highlighting::{Color, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub fn render_pretty_json(json: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set.find_syntax_by_extension("json").unwrap();

    let mut theme = theme_set.themes["base16-eighties.dark"].clone();
    theme.settings.background = Some(Color {
        r: 16,
        g: 24,
        b: 39,
        a: 0xFF,
    });
    let pretty_json = jsonxf::pretty_print(json).unwrap();
    highlighted_html_for_string(&*pretty_json, &syntax_set, syntax, &theme).unwrap()
}