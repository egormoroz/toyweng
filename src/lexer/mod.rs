pub mod token;
pub use token::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerError<'a> {
    UnknownLexeme(&'a str),
}

pub type LexerResult<'a> = Result<Token<'a>, LexerError<'a>>;


///just a token stream
#[derive(Debug, Clone, Copy)]
pub struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn peek(&mut self) -> LexerResult<'a> {
        self.source = self.source.trim_start();
        let mut it = self.source.chars();
        let (ch, ch2) = (it.next(), it.next());
        let ch = match ch {
            Some(ch) => ch,
            None => return Ok(Token::EOF)
        };

        match ch {
            '<' => match ch2 {
                Some('/') => Ok(Token::CloseTagStart),
                _ => Ok(Token::OpenTagStart),
            }
            '>' => Ok(Token::TagEnd),
            '"' => Ok(Token::Quote),
            '=' => Ok(Token::Equals),
            ch if ch.is_alphabetic() => Ok(Token::Identifier("")),
            _ => Err(LexerError::UnknownLexeme(self.source)),
        }
    }

    pub fn take(&mut self, t: Token<'a>) -> Token<'a> {
        use Token::*;
        match t {
            EOF => t,
            Identifier(_) => Token::Identifier(self.take_identifier()),
            CloseTagStart => {
                self.source = &self.source[2..];
                t
            }
            _ => {
                self.source = &self.source[1..];
                t
            }
        }
    }

    pub fn next(&mut self) -> LexerResult<'a> {
        self.peek().and_then(|t| Ok(self.take(t)))
    }


    pub fn text_till(&mut self, c: char) -> &'a str {
        self.source = self.source.trim_start();
        let n = self.source.find(c)
            .unwrap_or(self.source.len());
        self.cut_front(n).trim_end()
    }

    ///return remainder of source
    pub fn remainder(&self) -> &'a str {
        self.source
    }

    fn take_identifier(&mut self) -> &'a str {
        let n = self.source.find(
            |c: char| !c.is_alphanumeric()
        ).unwrap_or(self.source.len());
        self.cut_front(n)
    }

    ///cut off first n utf-8 bytes and return them
    fn cut_front(&mut self, n: usize) -> &'a str {
        let result = &self.source[..n];
        self.source = &self.source[n..];
        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn peek_simple() {
        let tests = [
            (OpenTagStart, "<"), (CloseTagStart, "</"),
            (TagEnd, ">"), (Quote, "\""), (Equals, "="),
            (Identifier(""), "asdf"), (EOF, ""),
        ];
        for (t, s) in tests {
            assert_eq!(Ok(t), Lexer::new(s).peek());
        }
    }

    #[test]
    fn take_simple_eof() {
        let tests = [
            (OpenTagStart, "<"), (CloseTagStart, "</"),
            (TagEnd, ">"), (Quote, "\""), (Equals, "="),
            (EOF, ""),
        ];
        for (t, s) in tests {
            let mut lx = Lexer::new(s);
            assert_eq!(Ok(t), lx.next());
            assert_eq!(Ok(EOF), lx.next());
        }
    }

    #[test]
    fn take_ident_eof() {
        let tests = [
            (Identifier("a"), "a"), (Identifier("asdf"), "asdf"),
            (Identifier("asdf123"), "asdf123")
        ];
        for (t, s) in tests {
            let mut lx = Lexer::new(s);
            assert_eq!(Ok(t), lx.next());
            assert_eq!(Ok(Token::EOF), lx.next());
        }
    }

    #[test]
    fn ignore_whitespace() {
        let s = "\n<html>   </html>   ";
        let tokens = [
            OpenTagStart, Identifier("html"), TagEnd,
            CloseTagStart, Identifier("html"), TagEnd,
            EOF
        ];
        let mut lx = Lexer::new(s);
        for t in tokens {
            assert_eq!(Ok(t), lx.next());
        }
    }

    #[test]
    fn text_till() {
        let mut lx = Lexer::new("asdf</");
        assert_eq!("asdf", lx.text_till('<'));
        assert_eq!(Ok(CloseTagStart), lx.next());
        assert_eq!(Ok(EOF), lx.next());
    }

    fn get_ident<'a>(lx: &mut Lexer<'a>) -> &'a str {
        match lx.next() {
            Ok(Identifier(ident)) => ident,
            other => panic!("expected identifier, got {:?}", other),
        }
    }

    fn tag<'a>(lx: &mut Lexer<'a>, open: bool) -> &'a str {
        let t = if open { OpenTagStart } else { CloseTagStart };
        assert_eq!(Ok(t), lx.next());
        let ident = get_ident(lx);
        assert_eq!(Ok(TagEnd), lx.next());

        ident
    }

    fn attrib<'a>(lx: &mut Lexer<'a>) -> (&'a str, &'a str) {
        let name = get_ident(lx);
        assert_eq!(Ok(Equals), lx.next());
        assert_eq!(Ok(Quote), lx.next());
        let value = lx.text_till('"');
        assert_eq!(Ok(Quote), lx.next());

        (name, value)
    }

    #[test]
    fn simple_html_doc() {
        let s = r#"
<html>
    <body>
        Hello world!
        <a href="google.com">google</a>
    </body>
</html>
        "#;
        let mut lx = Lexer::new(s);
        assert_eq!("html", tag(&mut lx, true));
        assert_eq!("body", tag(&mut lx, true));
        assert_eq!("Hello world!", lx.text_till('<'));

        assert_eq!(Ok(OpenTagStart), lx.next());
        assert_eq!("a", get_ident(&mut lx));
        assert_eq!(("href", "google.com"), attrib(&mut lx));
        assert_eq!(Ok(TagEnd), lx.next());
        assert_eq!("google", lx.text_till('<'));
        assert_eq!("a", tag(&mut lx, false));

        assert_eq!("body", tag(&mut lx, false));
        assert_eq!("html", tag(&mut lx, false));
    }
}
