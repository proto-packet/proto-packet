use code_gen::rust::WithComments;

/// Adds the schema `comments` to the `result` as doc comments.
///
/// # Note
/// - A comment is text, so it is escaped for rustdoc to show it as it is written.
/// - The text between two backticks is code.
/// - The blank lines at the start & the end are dropped.
pub(crate) fn add_docs<R>(result: &mut R, comments: &[String])
where
    R: WithComments,
{
    let lines: Vec<String> = comments
        .iter()
        .map(|comment| escape_line(comment.replace('\t', " ").trim()))
        .collect();
    let start: usize = lines.iter().take_while(|line| line.is_empty()).count();
    let end: usize = lines.len()
        - lines
            .iter()
            .rev()
            .take_while(|line| line.is_empty())
            .count();
    for line in lines.iter().take(end).skip(start) {
        if line.is_empty() {
            result.add_comment("");
        } else {
            result.add_comment(format!(" {}", line));
        }
    }
}

/// A schema member line of a pps doc block, with its comments. (ex: `name: string = 1;`)
pub(crate) struct PpsLine<'a> {
    pub(crate) comments: &'a [String],
    pub(crate) line: String,
}

/// Adds the pps doc block of a type declaration to the `result`.
///
/// The block re-prints the schema declaration, since the generated fields are private & their
/// comments, tag numbers & optionality would not show in the docs otherwise.
///
/// # Note
/// - A tab in a comment is written as spaces, since clippy warns about a tab in a doc comment.
pub(crate) fn add_pps_docs<R>(
    result: &mut R,
    keyword: &str,
    type_name: &str,
    comments: &[String],
    members: &[PpsLine<'_>],
) where
    R: WithComments,
{
    result.add_comment(" ```pps");
    for comment in comments {
        result.add_comment(format!(" //{}", pps_comment(comment)));
    }
    result.add_comment(format!(" {} {} {{", keyword, type_name));
    for member in members {
        result.add_comment("");
        for comment in member.comments {
            result.add_comment(format!("     //{}", pps_comment(comment)));
        }
        result.add_comment(format!("     {}", member.line));
    }
    result.add_comment(" }");
    result.add_comment(" ```");
}

/// Escapes the `text` of a comment line, so that rustdoc shows it as it is written.
///
/// # Note
/// - A code span is kept as it is.
/// - A URL is put in angle brackets, since rustdoc warns about a bare URL.
fn escape_line(text: &str) -> String {
    let mut result: String = String::with_capacity(text.len());
    let mut rest: &str = text;
    let digits: usize = rest.bytes().take_while(u8::is_ascii_digit).count();
    let after_digits: &str = &rest[digits..];
    if digits > 0
        && after_digits.starts_with(['.', ')'])
        && after_digits[1..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
    {
        result.push_str(&rest[..digits]);
        result.push('\\');
        rest = after_digits;
    } else if rest.starts_with(['#', '-', '+', '=', ':']) {
        result.push('\\');
    }
    while let Some(next) = rest.chars().next() {
        let len: usize = if next == '`' {
            let run: usize = rest.bytes().take_while(|byte| *byte == b'`').count();
            if let Some(len) = code_span_len(rest, run) {
                result.push_str(&rest[..len]);
                len
            } else {
                result.push_str(&"\\`".repeat(run));
                run
            }
        } else if let Some(len) = url_len(rest) {
            result.push('<');
            result.push_str(&rest[..len]);
            result.push('>');
            len
        } else {
            let is_escaped: bool = match next {
                '\\' | '*' | '_' | '[' | ']' | '<' | '>' | '~' | '|' => true,
                '&' => rest[1..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '#'),
                _ => false,
            };
            if is_escaped {
                result.push('\\');
            }
            result.push(next);
            next.len_utf8()
        };
        rest = &rest[len..];
    }
    result
}

/// Gets the optional length of the code span that starts the `text` with a `run` of backticks.
///
/// # Note
/// - A code span ends at the next run of backticks of the same length.
fn code_span_len(text: &str, run: usize) -> Option<usize> {
    let bytes: &[u8] = text.as_bytes();
    let mut index: usize = run;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let len: usize = bytes[index..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if len == run {
                return Some(index + len);
            }
            index += len;
        } else {
            index += 1;
        }
    }
    None
}

/// Gets the optional length of the URL that starts the `text`.
///
/// # Note
/// - The punctuation after a URL is not a part of it, but a `)` that closes a `(` in the URL is.
fn url_len(text: &str) -> Option<usize> {
    let scheme: &str = ["https://", "http://"]
        .into_iter()
        .find(|scheme| text.starts_with(scheme))?;
    let mut len: usize = text
        .find(|c: char| c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | '`'))
        .unwrap_or(text.len());
    loop {
        let url: &str = &text[..len];
        match url.chars().next_back() {
            Some(')') if url.matches('(').count() >= url.matches(')').count() => break,
            Some('.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '\'' | '"') => len -= 1,
            _ => break,
        }
    }
    (len > scheme.len()).then_some(len)
}

/// Gets the `comment` as it is written in a pps doc block.
fn pps_comment(comment: &str) -> String {
    comment.trim_end().replace('\t', "    ")
}
