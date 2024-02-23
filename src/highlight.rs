use pulldown_cmark::CowStr;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

pub(super) fn code_to_html(code: &str, name: &Option<CowStr<'_>>) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["Solarized (dark)"];
    let syntax = if let Some(lang) = name {
        if let Some(syntax) = syntax_set.find_syntax_by_token(lang) {
            syntax
        } else {
            syntax_set.find_syntax_plain_text()
        }
    } else {
        syntax_set.find_syntax_plain_text()
    };
    highlighted_html_for_string(code, &syntax_set, syntax, theme).unwrap()
}
