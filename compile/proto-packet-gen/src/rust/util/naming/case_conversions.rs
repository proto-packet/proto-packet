use crate::rust::gen_rust::GenRust;

impl GenRust<'_> {
    //! Naming: Case Conversions

    /// Converts the `pascal_case` string to snake case.
    pub fn pascal_to_snake_case(self, pascal_case: &str) -> String {
        let chars: Vec<char> = pascal_case.chars().collect();
        let mut snake_case: String = String::with_capacity(pascal_case.len() + 4);
        for (i, &c) in chars.iter().enumerate() {
            if c.is_uppercase() && i != 0 {
                let prev_is_lower: bool = chars[i - 1].is_lowercase();
                let next_is_lower: bool = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
                if prev_is_lower || next_is_lower {
                    snake_case.push('_');
                }
            }
            for lc in c.to_lowercase() {
                snake_case.push(lc);
            }
        }
        snake_case
    }
}
