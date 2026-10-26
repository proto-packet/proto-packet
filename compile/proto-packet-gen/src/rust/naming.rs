//! The Rust names of schema names.

/// The Rust keywords that can be raw identifiers. (ex: `r#type`)
const RAW_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do", "dyn",
    "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let",
    "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return",
    "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use",
    "virtual", "where", "while", "yield",
];

/// The Rust keywords that cannot be raw identifiers.
const NON_RAW_KEYWORDS: &[&str] = &["crate", "self", "Self", "super"];

/// Converts the schema `name` to a Rust identifier.
///
/// Keywords become raw identifiers. (ex: `type` -> `r#type`) The keywords that cannot be raw
/// identifiers get a trailing underscore. (ex: `self` -> `self_`)
pub fn ident(name: &str) -> String {
    if RAW_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else if NON_RAW_KEYWORDS.contains(&name) {
        format!("{}_", name)
    } else {
        name.to_string()
    }
}

/// Converts the schema type `name` to snake case. (ex: `HttpServer` -> `http_server`)
///
/// An underscore is added before each word that starts with an uppercase letter, unless the
/// name already has one there. (ex: `My_Type` -> `my_type`, `HTTPServer` -> `http_server`)
pub fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut snake_case: String = String::with_capacity(name.len() + 4);
    for (i, c) in chars.iter().copied().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            let prev: char = chars[i - 1];
            let next_is_lowercase: bool = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
            let starts_word: bool = prev.is_ascii_lowercase()
                || prev.is_ascii_digit()
                || (prev.is_ascii_uppercase() && next_is_lowercase);
            if starts_word {
                snake_case.push('_');
            }
        }
        snake_case.push(c.to_ascii_lowercase());
    }
    snake_case
}
