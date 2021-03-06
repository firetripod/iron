use std::fs;
use iron::token::Token as Token;

#[allow(dead_code)]
pub fn hello() {
    println!("Hello from the lexer!");
}

#[allow(dead_code)]
fn lexer_panic() {
    panic!("error(lexer): Expected character.");
}

#[allow(dead_code)]
/// When we reach a non-alphanumeric symbol (e.g. `;`, `:`, `.`), we must treat the previous
/// current token as a finalized token and push it to our result vector accordingly. This ensures
/// that we don't come across a parsing error if someone forgets to use spaces as intended.
fn push_token(current: &mut String, result: &mut Vec<Token>, token: Token) {
    if !current.is_empty() {
        let current_token = Token::Identifier(current.clone());
        result.push(current_token);
        current.clear();
    }
    result.push(token);
}

#[allow(dead_code)]
/// Public interface that takes a file path and returns a vector of Tokens corresponding to that
/// input Iron file. We leverage a peekable list of characters to handle complex tokens like
/// `->`, `=>`, `+=`, and so on.
///
/// # Examples
///
/// ```
/// let tokenized_input: Vec<Token> = iron::lexer::tokens_from_path("example.fe");
/// ```
pub fn tokens_from_path(path: &str) -> Vec<Token> {
    let mut current = String::new();
    let mut result: Vec<Token> = Vec::new();
    let data = fs::read_to_string(path).expect("error(lexer): Unable to read file.");

    let mut chars = data.chars().peekable();
    while chars.peek() != None {
        match chars.next() {
            Some(x) => {
                match x {
                    '+' => push_token(&mut current, &mut result, Token::Plus),
                    ';' => push_token(&mut current, &mut result, Token::Semicolon),
                    ':' => push_token(&mut current, &mut result, Token::OfTypeIndicator),
                    '\'' => push_token(&mut current, &mut result, Token::SingleQuote),
                    ',' => push_token(&mut current, &mut result, Token::Comma),
                    '.' => push_token(&mut current, &mut result, Token::Dot),
                    '(' => push_token(&mut current, &mut result, Token::OpenParen),
                    ')' => push_token(&mut current, &mut result, Token::CloseParen),
                    '[' => push_token(&mut current, &mut result, Token::OpenBracket),
                    ']' => push_token(&mut current, &mut result, Token::CloseBracket),
                    '{' => push_token(&mut current, &mut result, Token::OpenBrace),
                    '}' => push_token(&mut current, &mut result, Token::CloseBrace),
                    '*' => push_token(&mut current, &mut result, Token::Asterisk),
                    '@' => push_token(&mut current, &mut result, Token::At),
                    '$' => push_token(&mut current, &mut result, Token::Template),
                    '<' => push_token(&mut current, &mut result, Token::LessThan),
                    '>' => push_token(&mut current, &mut result, Token::GreaterThan),
                    // '_' => push_token(&mut current, &mut result, Token::Underscore),

                    // TODO: Handle all escapable characters, not just the backslash.
                    //       (e.g.) double quote, template, etc.
                    '\\' => push_token(&mut current, &mut result, Token::BackSlash),
                    '/' => {
                        if chars.peek() == Some(&'/') {
                            let mut is_doc = false;
                            let mut comment = String::new();
                            let _begin_comment = chars.next();
                            if chars.peek() == Some(&'/') {
                                is_doc = true;
                                let _begin_document_comment = chars.next();
                            }
                            if chars.peek() == Some(&' ') {
                                let _consume_space = chars.next();
                            }
                            while chars.peek() != Some(&'\n') {
                                match chars.next() {
                                    Some(c) => comment.push(c),
                                    None => lexer_panic()
                                }
                            }
                            if is_doc {
                                push_token(&mut current, &mut result, Token::DocumentComment(comment));
                            } else {
                                push_token(&mut current, &mut result, Token::LineComment(comment));
                            }
                        } else {
                            push_token(&mut current, &mut result, Token::ForwardSlash);
                        }
                    },
                    '"' => {
                        let mut string_end = false;
                        let mut is_template = false;
                        let mut string_literal = String::new();
                        while !string_end {
                            match chars.next() {
                                Some(c) => {
                                    if c == '"' {
                                        string_end = true;
                                    } else if c == '$' {
                                        string_literal.push(c);
                                        if chars.peek() == Some(&'{') {
                                            is_template = true;
                                        }
                                    } else {
                                        string_literal.push(c);
                                    }
                                },
                                None => lexer_panic()
                            }
                        }
                        if is_template {
                            // TODO: Parse the templates and convert them to vectors of Tokens
                            // that can be evaluated by the parser.
                            push_token(&mut current, &mut result, Token::TemplateString(string_literal));
                        } else {
                            push_token(&mut current, &mut result, Token::StringLiteral(string_literal));
                        }
                    },
                    '=' => {
                        if chars.peek() == Some(&'=') {
                            result.push(Token::EqualityOperator);
                            chars.next();
                        } else if chars.peek() == Some(&'>') {
                            result.push(Token::FatArrow);
                            chars.next();
                        } else {
                            result.push(Token::Equals);
                        }
                    },
                    '-' => {
                        if chars.peek() == Some(&'>') {
                            result.push(Token::ReturnTypeIndicator);
                            chars.next();
                        } else {
                            result.push(Token::Dash);
                        }
                    },
                    _ if x.is_whitespace() => {
                        if !current.is_empty() {
                            let mut current_token: Token;
                            match &current[..] {
                                "let" => current_token = Token::Let,
                                "type" => current_token = Token::Type,
                                "import" => current_token = Token::Import,
                                "function" => current_token = Token::Function,
                                "protocol" => current_token = Token::Protocol,
                                "public" => current_token = Token::Public,
                                "async" => current_token = Token::Async,
                                _ => current_token = Token::Identifier(current.clone())
                            }
                            result.push(current_token);
                            current.clear();
                        }
                    },
                    _ => {
                        current.push(x);
                    }
                }
            }
            _ => lexer_panic()
        }
    }

    result.push(Token::EndOfFile);
    result
}
