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

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => {
                in_single = !in_single;
            }

            '"' if !in_single => {
                in_double = !in_double;
            }

            '#' if !in_single && !in_double => {
                break;
            }

            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }

            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push('\\');
                }
            }

            _ => {
                current.push(c);
            }
        }
    }

    if in_single || in_double {
        return TokenizeState::Incomplete;
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    TokenizeState::Complete(tokens)
}