
use crate::{
    dom::*, 
    lexer::*,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    LexerError(LexerError<'a>),
    UnexpectedToken {
        expected: Token<'a>,
        got: Token<'a>,
        src: &'a str,
    },
    TagMismatch {
        opened: &'a str,
        closed: &'a str,
    }
}

pub type ParseResult<'a> = Result<Node<'a>, ParseError<'a>>;

pub fn parse<'a>(lx: Lexer<'a>) -> ParseResult<'a> {
    Parser { lx }.node()
}

struct Parser<'a> {
    lx: Lexer<'a>,
}

impl<'a> Parser<'a> {
    fn nodes(&mut self) 
        -> Result<Vec<Node<'a>>, ParseError<'a>> 
    {
        let mut ns = vec![];
        loop {
            if let Ok(Token::CloseTagStart) = self.peek() {
                break;
            }
            ns.push(self.node()?);
        }

        Ok(ns)
    }

    fn node(&mut self) -> ParseResult<'a> {
        match self.peek() {
            Ok(Token::OpenTagStart) => self.element(),
            _ => Ok(text(self.lx.text_till('<'))) //consider unknown lexeme as text
        }         
    }

    fn element(&mut self) -> ParseResult<'a> {
        self.eat(Token::OpenTagStart)?;
        let tag_name = self.eat_ident()?;
        let attrs = self.attributes()?;
        self.eat(Token::TagEnd)?;

        let children = self.nodes()?;

        self.eat(Token::CloseTagStart)?;
        let close_tag_name = self.eat_ident()?;
        self.eat(Token::TagEnd)?;

        if tag_name != close_tag_name {
            Err(ParseError::TagMismatch {
                opened: tag_name,
                closed: close_tag_name
            })
        } else {
            Ok(elem(tag_name, attrs, children))
        }
    }

    fn attributes(&mut self) -> Result<AttrMap<'a>, ParseError<'a>> {
        let mut attrs = AttrMap::new();

        loop {
            if let Token::TagEnd = self.peek()? {
                break;
            }
            let (k, v) = self.attribute()?;
            attrs.insert(k, v);
        }

        Ok(attrs)
    }

    fn attribute(&mut self) -> Result<(&'a str, &'a str), ParseError<'a>> {
        let attr_name = self.eat_ident()?;
        if let Token::Equals = self.peek()? {
            self.eat(Token::Equals)?;

            self.eat(Token::Quote)?;
            let attr_value = self.lx.text_till('"');
            self.eat(Token::Quote)?;

            Ok((attr_name, attr_value))
        } else {
            Ok((attr_name, ""))
        }
    }

    fn eat(&mut self, expected: Token<'a>) -> Result<Token<'a>, ParseError<'a>> {
        let src = self.lx.remainder();
        let got = self.next_token()?;
        if got.same_type(&expected) {
            Ok(got)
        } else {
            Err(ParseError::UnexpectedToken { expected, got, src })
        }
    }

    fn eat_ident(&mut self) -> Result<&'a str, ParseError<'a>> {
        let src = self.lx.remainder();
        match self.next_token()? {
            Token::Identifier(id) => Ok(id),
            got => Err(ParseError::UnexpectedToken {
                got,
                expected: Token::Identifier(""),
                src
            })
        }
    }

    fn next_token(&mut self) -> Result<Token<'a>, ParseError<'a>> {
        self.lx.next().map_err(|e| ParseError::LexerError(e))
    }

    fn peek(&mut self) -> Result<Token<'a>, ParseError<'a>> {
        self.lx.peek().map_err(|e| ParseError::LexerError(e))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    //https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal
    macro_rules! collection {
        ($($k:expr => $v:expr),* $(,)?) => {{
            use std::iter::{Iterator, IntoIterator};
            Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
        }};
    }

    #[test]
    fn lonely_html_tag() {
        let expected = elem(
            "html", 
            AttrMap::new(), 
            vec![]
        );
        let src = "<html></html>";
        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn nested_single() {
        let expected = elem(
            "html", 
            AttrMap::new(), 
            vec![elem(
                "head",
                AttrMap::new(),
                vec![]
            )]
        );
        let src = "<html><head></head></html>";

        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn nested_multiple() {
        let expected = elem(
            "html", 
            AttrMap::new(), 
            vec![
                elem("head", AttrMap::new(), vec![]),
                elem("body", AttrMap::new(), vec![]),
                elem("bruv", AttrMap::new(), vec![]),
            ]
        );
        let src = r#"
        <html>
            <head></head>
            <body></body>
            <bruv></bruv>
        </html>        
        "#;

        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn nested_deep() {
        let expected = elem(
            "html",
            AttrMap::new(),
            vec![elem(
                "body", 
                AttrMap::new(),
                vec![elem(
                    "div", 
                    AttrMap::new(), 
                    vec![elem(
                        "div",
                        AttrMap::new(),
                        vec![]
                    )]
                )]
            )]
        );
        let src = r#"
        <html>
            <body>
                <div>
                    <div>
                    </div>
                </div>
            </body>
        </html>
        "#;

        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn attrib_single() {
        let expected = elem(
            "tag",
            collection! { "attrib" => "attr val" },
            vec![]
        );
        let src = "<tag attrib=\"attr val\"></tag>";

        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn attrib_multiple() {
        let expected = elem(
            "image",
            collection! { 
                "src" => "image.png" ,
                "width" => "640",
                "height" => "480",
            },
            vec![]
        );
        let src = r#"
            <image src="image.png" width="640" height="480">
            </image>
        "#; //tags like image aren't supported yet

        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn nested_text() {
        let expected = elem(
            "body",
            AttrMap::new(),
            vec![
                text("uwu!"),
                elem("p", AttrMap::new(), vec![text("rawr :3")])
            ]
        );
        let src = r#"
        <body>
            uwu!
            <p>rawr :3</p>
        </body>
        "#;
        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }

    #[test]
    fn simple_html_doc() {
        let expected = elem(
            "html",
            AttrMap::new(),
            vec![elem(
                "body",
                AttrMap::new(),
                vec![
                    elem("h1", AttrMap::new(), vec![text("Title")]),
                    elem(
                        "div",
                        collection! { 
                            "id" => "main", 
                            "class" => "test",
                        },
                        vec![elem(
                            "p",
                            AttrMap::new(),
                            vec![
                                text("Hello"),
                                elem("em", AttrMap::new(), vec![text("world")]),
                                text("!")
                            ]
                        )]
                    )
                ]
            )]
        );

        let src = r#"
            <html>
                <body>
                    <h1>Title</h1>
                    <div id="main" class="test">
                        <p>Hello <em>world</em>!</p>
                    </div>
                </body>
            </html>
        "#;

        assert_eq!(Ok(expected), parse(Lexer::new(src)));
    }
}
