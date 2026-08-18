pub enum TokenizeState {
    Complete(Vec<String>),
    Incomplete,
}

pub fn tokenize(input: &str) -> TokenizeState {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;

    while let Some(c) = chars.next() {
        match c {
            // Single quote
            '\'' if !in_double && !in_backtick => {
                in_single = !in_single;
            }

            // Double quote
            '"' if !in_single && !in_backtick => {
                in_double = !in_double;
            }

            // Backtick
            '`' if !in_single && !in_double => {
                in_backtick = !in_backtick;
            }

            // Comment
            '#' if !in_single && !in_double && !in_backtick => {
                break;
            }

            // Whitespace separates tokens only outside quotes
            c if c.is_whitespace()
                && !in_single
                && !in_double
                && !in_backtick =>
            {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }

            // Backslash escapes the next character
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push('\\');
                }
            }

            // Normal character
            _ => {
                current.push(c);
            }
        }
    }

    // An unclosed quote/backtick means the command is incomplete
    if in_single || in_double || in_backtick {
        return TokenizeState::Incomplete;
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    TokenizeState::Complete(tokens)
}