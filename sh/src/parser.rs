/// Simple shell tokenizer supporting single and double quotes.
pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
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
            _ => current.push(c),
        }
    }

    if in_single || in_double {
        return Err("unclosed quote".to_string());
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_whitespace() {
        assert_eq!(tokenize("ls -la").unwrap(), vec!["ls", "-la"]);
    }

    #[test]
    fn double_quotes() {
        assert_eq!(
            tokenize(r#"echo "Hello There""#).unwrap(),
            vec!["echo", "Hello There"]
        );
    }

    #[test]
    fn single_quotes() {
        assert_eq!(tokenize("echo 'a b'").unwrap(), vec!["echo", "a b"]);
    }
}
