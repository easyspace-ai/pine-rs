pub mod token;

pub use token::{LexError, Span, Token};

use logos::Logos;

/// A lexed token with its span
pub type SpannedToken = (Token, Span);

/// Lexer for Pine Script v6
pub struct Lexer;

impl Lexer {
    /// Lex the source code into a vector of tokens with spans
    /// Includes INDENT/DEDENT tokens for significant whitespace
    pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
        let mut tokens = Vec::new();
        let mut logos_lexer = Token::lexer(source);
        let _indent_stack: Vec<usize> = vec![0];
        let mut _last_line_indent = 0;

        while let Some(result) = logos_lexer.next() {
            match result {
                Ok(Token::Newline) => {
                    // Track position for indentation processing
                    let span = Span::new(logos_lexer.span().start, logos_lexer.span().end);
                    tokens.push((Token::Newline, span));

                    // Look ahead to check indentation of next line
                    let current_pos = logos_lexer.span().end;
                    let remaining = &source[current_pos..];

                    // Find next non-empty line
                    for line in remaining.lines() {
                        let trimmed = line.trim_start();
                        if !trimmed.is_empty() && !trimmed.starts_with("//") {
                            let _indent = line.len() - trimmed.len();
                            break;
                        }
                    }
                }
                Ok(token) => {
                    let span = Span::new(logos_lexer.span().start, logos_lexer.span().end);
                    tokens.push((token, span));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(tokens)
    }

    /// Lex with indentation processing (generates INDENT/DEDENT tokens)
    pub fn lex_with_indentation(source: &str) -> Result<Vec<SpannedToken>, LexError> {
        let mut tokens = Vec::new();
        let mut indent_stack: Vec<usize> = vec![0];

        // Track absolute byte offsets so spans are stable across the full source (required for
        // call-site interning and multi-line error locations). `lex_line` alone is line-local.
        let mut line_start: usize = 0;
        while line_start <= source.len() {
            let line_end = source[line_start..]
                .find('\n')
                .map(|i| line_start + i)
                .unwrap_or(source.len());
            let raw_line = &source[line_start..line_end];
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

            let trimmed = line.trim_start();

            // Skip empty lines and comment-only lines
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                // Calculate indentation
                let indent = line.len() - trimmed.len();
                let current_indent = *indent_stack.last().unwrap_or(&0);

                // Handle indentation changes
                if indent > current_indent {
                    indent_stack.push(indent);
                    tokens.push((Token::Indent, Span::new(0, 0)));
                } else if indent < current_indent {
                    while indent < *indent_stack.last().unwrap_or(&0) {
                        indent_stack.pop();
                        tokens.push((Token::Dedent, Span::new(0, 0)));
                    }
                    if indent != *indent_stack.last().unwrap_or(&0) {
                        return Err(LexError);
                    }
                }

                let payload_offset = line_start + indent;
                let line_tokens = Self::lex_line(trimmed)?;
                for (tok, mut sp) in line_tokens {
                    sp.start += payload_offset;
                    sp.end += payload_offset;
                    tokens.push((tok, sp));
                }
                tokens.push((Token::Newline, Span::new(line_end, line_end)));
            }

            if line_end >= source.len() {
                break;
            }
            line_start = line_end + 1;
        }

        // Final dedents
        while indent_stack.len() > 1 {
            indent_stack.pop();
            tokens.push((Token::Dedent, Span::new(0, 0)));
        }

        Ok(tokens)
    }

    /// Lex a single line (without indentation)
    fn lex_line(line: &str) -> Result<Vec<SpannedToken>, LexError> {
        let mut tokens = Vec::new();
        let mut logos_lexer = Token::lexer(line);

        while let Some(result) = logos_lexer.next() {
            match result {
                Ok(token) => {
                    let span = Span::new(logos_lexer.span().start, logos_lexer.span().end);
                    tokens.push((token, span));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(tokens)
    }
}

/// Process indentation and insert virtual INDENT/DEDENT/NEWLINE tokens
///
/// Pine Script uses Python-style significant indentation. This function
/// processes raw tokens and inserts virtual tokens for:
/// - NEWLINE: At each line break
/// - INDENT: When indentation level increases
/// - DEDENT: When indentation level decreases (one per level)
pub fn process_indentation(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut result = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0]; // Start with 0 indentation

    for line in source.lines() {
        // Skip empty lines and comment-only lines for indentation purposes
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Calculate indentation (number of leading spaces/tabs)
        let indent = line.len() - trimmed.len();
        let current_indent = *indent_stack.last().unwrap_or(&0);

        if indent > current_indent {
            // Increased indentation
            indent_stack.push(indent);
            result.push((Token::Indent, Span::new(0, 0)));
        } else if indent < current_indent {
            // Decreased indentation - pop until we match
            while indent < *indent_stack.last().unwrap_or(&0) {
                indent_stack.pop();
                result.push((Token::Dedent, Span::new(0, 0)));
            }

            // If we don't match exactly, it's an indentation error
            if indent != *indent_stack.last().unwrap_or(&0) {
                return Err(LexError);
            }
        }

        // Add newline token
        result.push((Token::Newline, Span::new(0, 0)));
    }

    // Add dedents for any remaining indentation levels
    while indent_stack.len() > 1 {
        indent_stack.pop();
        result.push((Token::Dedent, Span::new(0, 0)));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let source = "var x = 42";
        let tokens = Lexer::lex(source).unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].0, Token::Var));
        assert!(matches!(tokens[1].0, Token::Ident(ref s) if s == "x"));
        assert!(matches!(tokens[2].0, Token::Assign));
        assert!(matches!(tokens[3].0, Token::Int(42)));
    }

    #[test]
    fn test_lexer_with_newlines() {
        let source = "var x = 42\nvar y = 100";
        let tokens = Lexer::lex(source).unwrap();

        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Newline)));
    }
}
