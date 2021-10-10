#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    OpenTagStart, //'<'
    CloseTagStart, //"</"
    TagEnd, //'>'
    Quote, //'"'
    Equals, //'='
    Identifier(&'a str), //any alphanumeric word, e.g. 'cat12'
    EOF //end of file
}

impl<'a> Token<'a> {
    pub fn same_type(&self, other: &Token) -> bool {
        use std::mem::discriminant;
        discriminant(self) == discriminant(other)
    }
}
