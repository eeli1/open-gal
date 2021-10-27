use crate::constants::{AND, NOT, OR, XOR};
use crate::parser::error::ParsingError;
use crate::parser::token::{Token, TokenType};

pub struct Lexer<'a> {
    data: &'a Vec<String>,
    line_index: usize,
    char_index: usize,
    current_line: String,
    current_char: char,
    tokens: Vec<Token>,
    eof: bool, // end of file
    eol: bool, // end of line by last char
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a Vec<String>) -> Self {
        let current_line = data[0].clone();
        if let Some(current_char) = current_line.clone().chars().nth(0) {
            Self {
                data,
                line_index: 0,
                char_index: 0,
                current_line,
                current_char,
                tokens: Vec::new(),
                eof: false,
                eol: false,
            }
        } else {
            panic!("can not initilize lexer because data.len() == 0");
        }
    }

    fn next(&mut self) {
        if self.eof {
            // TODO: make lexing error
            panic!("lexer has reached end of file on next char available");
        }

        if self.eol {
            if self.data.len() - 1 == self.line_index {
                self.eof = true;
                self.eol = true;
            } else {
                self.line_index += 1;
                self.char_index = 0;
                self.current_line = self.data[self.line_index].clone();
                self.current_char = self.current_line.chars().nth(0).unwrap();
                // if empty line
                self.eol = self.current_line.len() == 1;
            }
            return;
        }

        if self.current_line.len() - 2 == self.char_index {
            self.eol = true;
        }

        self.char_index += 1;
        self.current_char = self.current_line.chars().nth(self.char_index).unwrap();
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, ParsingError> {
        while !self.eof {
            if self.lex_char()? {
                continue;
            }

            if self.lex_num()? {
                continue;
            }

            if self.lex_identifier()? {
                continue;
            }

            if self.lex_arrow()? {
                continue;
            }

            if self.lex_comment()? {
                continue;
            }

            let err = ParsingError::from_token(
                Token {
                    begin_line: self.line_index,
                    begin_char: self.char_index,
                    len_char: 1,
                    len_line: 1,
                    token_type: TokenType::Unknown,
                },
                format!("unexpected character <{}>", self.current_char),
                self.data.clone(),
            );
            return Err(err);
        }

        return Ok(self.tokens.clone());
    }

    fn lex_comment(&mut self) -> Result<bool, ParsingError> {
        if self.current_char != '/' {
            return Ok(false);
        }

        let is_multiline;
        let begin_line = self.line_index;
        let begin_char = self.char_index;
        let mut comment = String::new();

        comment.push(self.current_char);
        self.next();

        if !self.eof {
            match self.current_char {
                '*' => is_multiline = true,
                '/' => is_multiline = false,
                _ => {
                    let err = ParsingError::from_token(
                        Token {
                            begin_line: self.line_index,
                            begin_char: self.char_index,
                            len_char: 1,
                            len_line: 1,
                            token_type: TokenType::Unknown,
                        },
                        format!(
                            "unexpected character expected <*, /> got <{}>",
                            self.current_char
                        ),
                        self.data.clone(),
                    );
                    return Err(err);
                }
            };
        } else {
            let err = ParsingError::from_token(
                Token {
                    begin_line: self.line_index,
                    begin_char: self.char_index,
                    len_char: 1,
                    len_line: 1,
                    token_type: TokenType::Unknown,
                },
                "unexpected line braek expected <*, /> got <new line>".to_string(),
                self.data.clone(),
            );
            return Err(err);
        }
        comment.push(self.current_char);
        self.next();

        if is_multiline {
            let mut last_star = false;
            loop {
                if !self.eof {
                    if self.current_char == '/' && last_star {
                        comment.push(self.current_char);
                        self.next();
                        break;
                    }

                    last_star = self.current_char == '*';
                    comment.push(self.current_char);
                    self.next();
                } else {
                    let err = ParsingError::from_token(
                        Token {
                            begin_line: self.line_index,
                            begin_char: self.char_index,
                            len_char: 1,
                            len_line: 1,
                            token_type: TokenType::Unknown,
                        },
                        format!(
                            "unexpected character expected <*, /> got <{}>",
                            self.current_char
                        ),
                        self.data.clone(),
                    );
                    return Err(err);
                }
            }
        } else {
            while !self.eol {
                comment.push(self.current_char);
                self.next();
            }
        }

        self.tokens.push(Token {
            begin_line,
            begin_char,
            len_char: comment.len(),
            len_line: self.line_index - begin_line + 1,
            token_type: TokenType::Ignore(Some(comment)),
        });
        Ok(true)
    }

    fn lex_identifier(&mut self) -> Result<bool, ParsingError> {
        if !Self::is_letter(self.current_char) {
            return Ok(false);
        }

        let mut name = String::new();
        let begin_char = self.char_index;

        while !self.eol {
            if Self::is_letter(self.current_char)
                || Self::is_digit(self.current_char)
                || self.current_char == '_'
            {
                name.push(self.current_char);
                self.next();
            } else {
                break;
            }
        }

        let token_type = match name.as_ref() {
            "pin" => TokenType::Pin,
            "table" => TokenType::Table,
            "count" => TokenType::Count,
            "fill" => TokenType::Fill,
            "dff" => TokenType::Dff,
            _ => TokenType::Identifier(name.clone()),
        };

        self.tokens.push(Token {
            begin_char,
            begin_line: self.line_index,
            len_char: name.len(),
            len_line: 1,
            token_type,
        });

        Ok(true)
    }

    fn lex_arrow(&mut self) -> Result<bool, ParsingError> {
        if self.current_char == '-' {
            self.next();
            if self.current_char == '>' {
                self.tokens.push(Token {
                    begin_line: self.line_index,
                    begin_char: self.char_index - 1,
                    len_char: 2,
                    len_line: 1,
                    token_type: TokenType::Arrow,
                });
                self.next();
                Ok(true)
            } else {
                let err = ParsingError::from_token(
                    Token {
                        begin_line: self.line_index,
                        begin_char: self.char_index,
                        len_char: 2,
                        len_line: 1,
                        token_type: TokenType::Unknown,
                    },
                    format!(
                        "unexpected char expected <{}> got <{}>",
                        '>', self.current_char
                    ),
                    self.data.clone(),
                );
                Err(err)
            }
        } else {
            Ok(false)
        }
    }

    fn lex_num(&mut self) -> Result<bool, ParsingError> {
        let begin_char = self.char_index;
        let first_char = self.current_char;
        if !Self::is_digit(first_char) {
            return Ok(false);
        }

        let mut num_chars = String::new();
        let begin_0 = first_char == '0';
        let mut is_bool = first_char == '0' || first_char == '1';
        num_chars.push(first_char);
        self.next();

        loop {
            if self.eol {
                break;
            }

            if Self::is_digit(self.current_char) {
                if !(self.current_char == '1' || self.current_char == '0') {
                    if begin_0 {
                        let err = ParsingError::from_token(
                            Token {
                                begin_char,
                                begin_line: self.line_index,
                                len_char: num_chars.len(),
                                len_line: 1,
                                token_type: TokenType::BoolTable(Vec::new()),
                            },
                            format!("expectet <0, 1> got <{}>", self.current_char),
                            self.data.clone(),
                        );
                        return Err(err);
                    }

                    if is_bool {
                        is_bool = false;
                    }
                }
                num_chars.push(self.current_char);
                self.next();
            } else {
                break;
            }
        }

        if is_bool {
            self.tokens.push(Token {
                begin_char,
                begin_line: self.line_index,
                len_char: num_chars.len(),
                len_line: 1,
                token_type: TokenType::BoolTable(num_chars.chars().map(|c| c == '1').collect()),
            });

            Ok(true)
        } else {
            let mut num_str = String::new();
            num_chars.chars().for_each(|c| num_str.push(c));

            let result: Result<isize, _> = num_str.parse();

            if result.is_err() {
                let err = ParsingError::from_token(
                    Token {
                        begin_char,
                        begin_line: self.line_index,
                        len_char: num_chars.len(),
                        len_line: 1,
                        token_type: TokenType::Number(0),
                    },
                    format!(
                        "parsing error while parsing number expectet <[0-9]> got <{}>",
                        num_str
                    ),
                    self.data.clone(),
                );
                return Err(err);
            }

            self.tokens.push(Token {
                begin_char,
                begin_line: self.line_index,
                len_char: num_chars.len(),
                len_line: 1,
                token_type: TokenType::Number(num_str.parse().unwrap()),
            });

            Ok(true)
        }
    }

    fn lex_char(&mut self) -> Result<bool, ParsingError> {
        let token_type_option = match self.current_char {
            AND => Some(TokenType::And),
            OR => Some(TokenType::Or),
            XOR => Some(TokenType::Xor),
            NOT => Some(TokenType::Not),

            '(' => Some(TokenType::RoundOpen),
            ')' => Some(TokenType::RoundClose),
            '{' => Some(TokenType::CurlyOpen),
            '}' => Some(TokenType::CurlyClose),
            '[' => Some(TokenType::SquareOpen),
            ']' => Some(TokenType::SquareClose),

            ',' => Some(TokenType::Comma),
            ';' => Some(TokenType::Semicolon),
            '=' => Some(TokenType::Equals),
            '.' => Some(TokenType::Dot),

            ' ' => Some(TokenType::Ignore(None)),
            '\t' => Some(TokenType::Ignore(None)),
            '\n' => Some(TokenType::Ignore(None)),

            _ => None,
        };

        if let Some(token_type) = token_type_option {
            self.tokens.push(Token {
                begin_line: self.line_index,
                len_char: 1,
                len_line: 1,
                begin_char: self.char_index,
                token_type,
            });
            self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn is_letter(c: char) -> bool {
        for l in "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz".chars() {
            if l == c {
                return true;
            }
        }
        return false;
    }

    fn is_digit(c: char) -> bool {
        for l in "0123456789".chars() {
            if l == c {
                return true;
            }
        }
        return false;
    }
}

// ---------------------------------------------------------------------- Tests ----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(vec: Vec<&str>) -> Vec<String> {
        vec.iter().map(|&s| format!("{}\n", s)).collect()
    }

    #[test]
    fn test_num() {
        let data = convert(vec!["123 010", "102 2 349645", "1 0 101 11"]);
        let mut lexer = Lexer::new(&data);
        let input = lexer.lex().unwrap();

        let output = Token::vec(vec![
            vec![
                TokenType::Number(123),
                TokenType::Ignore(None),
                TokenType::BoolTable(vec![false, true, false]),
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::Number(102),
                TokenType::Ignore(None),
                TokenType::Number(2),
                TokenType::Ignore(None),
                TokenType::Number(349645),
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::BoolTable(vec![true]),
                TokenType::Ignore(None),
                TokenType::BoolTable(vec![false]),
                TokenType::Ignore(None),
                TokenType::BoolTable(vec![true, false, true]),
                TokenType::Ignore(None),
                TokenType::BoolTable(vec![true, true]),
                TokenType::Ignore(None),
            ],
        ]);

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            assert_eq!(input[i], output[i], "at token <{}>", i);
        }
    }

    #[test]
    fn panic_num() {
        let data = convert(vec!["0103"]);
        let mut lexer = Lexer::new(&data);
        assert_eq!(
            lexer.lex(),
            Err(ParsingError::new(
                0,
                0,
                3,
                1,
                "expectet <0, 1> got <3>".to_string(),
                data
            ))
        );
    }

    #[test]
    fn lexer_next() {
        let data = convert(vec![
            "this is line one",
            "abc",
            "",
            "empty",
            "123 010",
            "102 2 349645",
        ]);
        let mut lexer = Lexer::new(&data);

        for (index, str_in) in data.iter().enumerate() {
            assert_eq!(index, lexer.line_index, "line_index");
            assert_eq!(str_in, &lexer.current_line);

            assert_eq!(false, lexer.eof, "not end eof");

            for (i, c) in str_in.chars().enumerate() {
                assert_eq!(i, lexer.char_index);
                assert_eq!(
                    c, lexer.current_char,
                    "line index {} char index {}",
                    index, i
                );

                if i == str_in.len() - 1 {
                    assert_eq!(
                        true,
                        lexer.eol,
                        "is eol at line {} len{}",
                        index,
                        str_in.len()
                    );
                }

                lexer.next();
            }
        }
        assert_eq!(true, lexer.eof);
    }

    #[test]
    fn chars() {
        let data = convert(vec![
            format!("{}{}", AND, OR).as_ref(),
            format!("{}{}{}{}", XOR, XOR, NOT, AND).as_ref(),
            "([",
            "{ ; }])",
            ".,==.,",
        ]);
        let mut lexer = Lexer::new(&data);

        let input = lexer.lex().unwrap();
        let output = Token::vec(vec![
            vec![TokenType::And, TokenType::Or, TokenType::Ignore(None)],
            vec![
                TokenType::Xor,
                TokenType::Xor,
                TokenType::Not,
                TokenType::And,
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::RoundOpen,
                TokenType::SquareOpen,
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::CurlyOpen,
                TokenType::Ignore(None),
                TokenType::Semicolon,
                TokenType::Ignore(None),
                TokenType::CurlyClose,
                TokenType::SquareClose,
                TokenType::RoundClose,
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::Dot,
                TokenType::Comma,
                TokenType::Equals,
                TokenType::Equals,
                TokenType::Dot,
                TokenType::Comma,
                TokenType::Ignore(None),
            ],
        ]);

        assert_eq!(
            input.len(),
            output.len(),
            "input output length dose not match"
        );
        for i in 0..input.len() {
            assert_eq!(input[i], output[i], "at token <{}>", i);
        }
    }

    #[test]
    fn doc_example() {
        let data = convert(vec![
            "pin in = 2;",
            "",
            format!("{}1010 // comment", AND).as_ref(),
        ]);
        let mut lexer = Lexer::new(&data);
        let input = lexer.lex().unwrap();

        let output = Token::vec(vec![
            vec![
                TokenType::Pin,
                TokenType::Ignore(None),
                TokenType::Identifier("in".to_string()),
                TokenType::Ignore(None),
                TokenType::Equals,
                TokenType::Ignore(None),
                TokenType::Number(2),
                TokenType::Semicolon,
                TokenType::Ignore(None),
            ],
            vec![TokenType::Ignore(None)],
            vec![
                TokenType::And,
                TokenType::BoolTable(vec![true, false, true, false]),
                TokenType::Ignore(None),
                TokenType::Ignore(Some("// comment".to_string())),
                TokenType::Ignore(None),
            ],
        ]);

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            assert_eq!(input[i], output[i], "at token <{}>", i);
        }
    }

    #[test]
    fn test_arrow() {
        let data = convert(vec![" ->"]);
        let mut lexer = Lexer::new(&data);
        let input = lexer.lex().unwrap();

        let output = Token::vec(vec![vec![
            TokenType::Ignore(None),
            TokenType::Arrow,
            TokenType::Ignore(None),
        ]]);

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            assert_eq!(input[i], output[i], "at token <{}>", i);
        }
    }

    #[test]
    fn test_identifier() {
        let data = convert(vec![
            "ab ab3",
            "c_f_g ",
            "pin table fill.count",
            "pin1",
            "dff",
        ]);
        let mut lexer = Lexer::new(&data);
        let input = lexer.lex().unwrap();

        let output = Token::vec(vec![
            vec![
                TokenType::Identifier("ab".to_string()),
                TokenType::Ignore(None),
                TokenType::Identifier("ab3".to_string()),
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::Identifier("c_f_g".to_string()),
                TokenType::Ignore(None),
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::Pin,
                TokenType::Ignore(None),
                TokenType::Table,
                TokenType::Ignore(None),
                TokenType::Fill,
                TokenType::Dot,
                TokenType::Count,
                TokenType::Ignore(None),
            ],
            vec![
                TokenType::Identifier("pin1".to_string()),
                TokenType::Ignore(None),
            ],
            vec![TokenType::Dff, TokenType::Ignore(None)],
        ]);

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            assert_eq!(input[i], output[i], "at token <{}>", i);
        }
    }
    #[test]
    fn test_comments() {
        let data = convert(vec![
            "// one line comment",
            "",
            "/*",
            "multi line comment",
            "*/",
        ]);
        let mut lexer = Lexer::new(&data);
        let input = lexer.lex().unwrap();

        let mut output = Vec::<Token>::new();

        output.push(Token {
            begin_line: 0,
            begin_char: 0,
            len_char: "// one line comment".len(),
            len_line: 1,
            token_type: TokenType::Ignore(Some("// one line comment".to_string())),
        });
        output.push(Token {
            begin_line: 0,
            begin_char: "// one line comment".len(),
            len_char: 1,
            len_line: 1,
            token_type: TokenType::Ignore(None),
        });

        output.push(Token {
            begin_line: 1,
            begin_char: 0,
            len_char: 1,
            len_line: 1,
            token_type: TokenType::Ignore(None),
        });

        output.push(Token {
            begin_line: 2,
            begin_char: 0,
            len_char: "/*\nmulti line comment\n*/".len(),
            len_line: 3,
            token_type: TokenType::Ignore(Some("/*\nmulti line comment\n*/".to_string())),
        });

        output.push(Token {
            begin_line: 4,
            begin_char: 2,
            len_char: 1,
            len_line: 1,
            token_type: TokenType::Ignore(None),
        });

        assert_eq!(input.len(), output.len());
        for i in 0..input.len() {
            assert_eq!(input[i], output[i]);
        }
    }
}