use crate::rust::GenRust;

impl GenRust<'_> {
    //! Naming: Mod Ident

    /// The rust keywords that are raw-escapable. (`crate`/`self`/`Self`/`super` cannot be
    /// raw identifiers, but they cannot be schema mod names either)
    const RUST_KEYWORDS: &'static [&'static str] = &[
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
        "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if",
        "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override",
        "priv", "pub", "ref", "return", "static", "struct", "trait", "true", "try", "type",
        "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ];

    /// Converts the `mod_name` to a rust identifier, raw-escaping the rust keywords.
    /// (ex: `move` -> `r#move`)
    pub fn mod_ident(mod_name: &str) -> String {
        if Self::RUST_KEYWORDS.contains(&mod_name) {
            format!("r#{mod_name}")
        } else {
            mod_name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::GenRust;

    #[test]
    fn mod_ident() {
        assert_eq!(GenRust::mod_ident("pokemon"), "pokemon");
        assert_eq!(GenRust::mod_ident("move"), "r#move");
        assert_eq!(GenRust::mod_ident("type"), "r#type");
        assert_eq!(GenRust::mod_ident("movement"), "movement");
    }
}
