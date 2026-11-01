//! A reader for PowerShell data files (`.psd1`), the restricted language `Import-PowerShellDataFile`
//! accepts, as far as a settings file uses it: hashtables `@{ key = value; … }`, arrays `@( … )`
//! and comma lists (which go on past a line break after a comma), single- and double-quoted
//! strings, bare words such as numbers, `$true` and `$false`, `#` and `<# … #>` comments. The
//! backup job keeps its settings so (EP-7); anything else is an error naming its line.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    /// A number, `$true`, `$false`, `$null` or another bare word, as written.
    Bare(String),
    List(Vec<Value>),
    /// Keys in the file's order.
    Map(Vec<(String, Value)>),
}

impl Value {
    /// The value of `key` in a hashtable, case aside, as PowerShell compares keys.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(entries) => entries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(text) | Value::Bare(text) => Some(text),
            _ => None,
        }
    }

    /// The items of a list; a single value is a list of one, as PowerShell unrolls it.
    pub fn items(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            one => std::slice::from_ref(one),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    OpenMap,
    OpenList,
    Close,
    CloseParen,
    Equals,
    Semicolon,
    Comma,
    Newline,
    Str(String),
    Bare(String),
}

/// Parses the file's one top-level value.
pub fn parse(text: &str) -> Result<Value, String> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, at: 0 };
    parser.skip(&[Token::Newline, Token::Semicolon]);
    let value = parser.element()?;
    parser.skip(&[Token::Newline, Token::Semicolon]);
    match parser.peek() {
        None => Ok(value),
        Some((token, line)) => Err(format!("line {line}: unexpected {token:?} after the value")),
    }
}

fn tokenize(text: &str) -> Result<Vec<(Token, usize)>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    let mut line = 1;
    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                tokens.push((Token::Newline, line));
                line += 1;
            }
            c if c.is_whitespace() => {}
            '#' => {
                while chars.peek().is_some_and(|&c| c != '\n') {
                    chars.next();
                }
            }
            '<' if chars.peek() == Some(&'#') => {
                chars.next();
                let start = line;
                let mut previous = ' ';
                loop {
                    match chars.next() {
                        Some('>') if previous == '#' => break,
                        Some(c) => {
                            if c == '\n' {
                                line += 1;
                            }
                            previous = c;
                        }
                        None => return Err(format!("line {start}: a <# comment is not closed")),
                    }
                }
            }
            '@' => match chars.next() {
                Some('{') => tokens.push((Token::OpenMap, line)),
                Some('(') => tokens.push((Token::OpenList, line)),
                _ => return Err(format!("line {line}: @ starts neither @{{ nor @(")),
            },
            '(' => tokens.push((Token::OpenList, line)),
            '}' => tokens.push((Token::Close, line)),
            ')' => tokens.push((Token::CloseParen, line)),
            '=' => tokens.push((Token::Equals, line)),
            ';' => tokens.push((Token::Semicolon, line)),
            ',' => tokens.push((Token::Comma, line)),
            '\'' | '"' => {
                let start = line;
                let mut text = String::new();
                loop {
                    match chars.next() {
                        // A doubled quote is the quote itself.
                        Some(q) if q == c && chars.peek() == Some(&c) => {
                            chars.next();
                            text.push(c);
                        }
                        Some(q) if q == c => break,
                        Some('`') if c == '"' => match chars.next() {
                            Some('n') => text.push('\n'),
                            Some('t') => text.push('\t'),
                            Some(other) => text.push(other),
                            None => break,
                        },
                        Some(other) => {
                            if other == '\n' {
                                line += 1;
                            }
                            text.push(other);
                        }
                        None => return Err(format!("line {start}: a string is not closed")),
                    }
                }
                tokens.push((Token::Str(text), start));
            }
            _ => {
                let mut word = String::from(c);
                while chars
                    .peek()
                    .is_some_and(|&c| !c.is_whitespace() && !"=;,(){}#'\"".contains(c))
                {
                    word.extend(chars.next());
                }
                tokens.push((Token::Bare(word), line));
            }
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&(Token, usize)> {
        self.tokens.get(self.at)
    }

    fn line(&self) -> usize {
        self.peek()
            .or(self.tokens.last())
            .map_or(1, |(_, line)| *line)
    }

    fn skip(&mut self, kinds: &[Token]) {
        while self.peek().is_some_and(|(token, _)| kinds.contains(token)) {
            self.at += 1;
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.peek().map(|(token, _)| token.clone());
        self.at += 1;
        token
    }

    /// One value: a hashtable, an array or a scalar.
    fn element(&mut self) -> Result<Value, String> {
        let line = self.line();
        match self.next() {
            Some(Token::OpenMap) => self.map(),
            Some(Token::OpenList) => self.list(),
            Some(Token::Str(text)) => Ok(Value::Str(text)),
            Some(Token::Bare(word)) => Ok(Value::Bare(word)),
            Some(other) => Err(format!("line {line}: a value cannot start with {other:?}")),
            None => Err(format!(
                "line {line}: the file ends where a value should be"
            )),
        }
    }

    /// One expression: an element, or elements joined by commas into a list.
    fn expression(&mut self) -> Result<Value, String> {
        let first = self.element()?;
        let mut items = vec![first];
        while self.peek().is_some_and(|(token, _)| *token == Token::Comma) {
            self.at += 1;
            self.skip(&[Token::Newline]);
            items.push(self.element()?);
        }
        Ok(if items.len() == 1 {
            items.remove(0)
        } else {
            Value::List(items)
        })
    }

    /// `@{` seen: `key = expression` entries split by line breaks or semicolons, up to `}`.
    fn map(&mut self) -> Result<Value, String> {
        let mut entries = Vec::new();
        loop {
            self.skip(&[Token::Newline, Token::Semicolon]);
            let line = self.line();
            let key = match self.next() {
                Some(Token::Close) => return Ok(Value::Map(entries)),
                Some(Token::Str(key) | Token::Bare(key)) => key,
                Some(other) => return Err(format!("line {line}: {other:?} is not a key")),
                None => return Err(format!("line {line}: a @{{ is not closed")),
            };
            if self.next() != Some(Token::Equals) {
                return Err(format!("line {line}: {key} has no = after it"));
            }
            self.skip(&[Token::Newline]);
            entries.push((key, self.expression()?));
        }
    }

    /// `@(` seen: statements split by line breaks or semicolons, up to `)`; a statement that is a
    /// list adds its items, as PowerShell unrolls the output of each.
    fn list(&mut self) -> Result<Value, String> {
        let mut items = Vec::new();
        loop {
            self.skip(&[Token::Newline, Token::Semicolon]);
            if self
                .peek()
                .is_some_and(|(token, _)| *token == Token::CloseParen)
            {
                self.at += 1;
                return Ok(Value::List(items));
            }
            if self.peek().is_none() {
                return Err(format!("line {}: a @( is not closed", self.line()));
            }
            match self.expression()? {
                Value::List(more) => items.extend(more),
                one => items.push(one),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETTINGS: &str = r#"# The settings of a scratch job.
@{
    # Where the archives go.
    Target              = 'E:\Archive\Files'
    Label               = "Scratch ""disk"""
    KeepDays            = 7
    Enabled             = $true
    <# A block comment
       over two lines. #>
    Sets                = @(
        @{ Name = 'NOTES'; Paths = @('NOTES') }
        @{ Name = 'CONFIG'; Paths = @('a.ini', 'b.ini',
                'tools\c.ini'); LooseFilesOf = @('WORK') }
        @{ Name = 'WORK'; Each = 'WORK' }
    )
    Single              = 'only'
    Commas              = 'x', 'y'
    Empty               = @()
}
"#;

    #[test]
    fn reads_a_settings_file() {
        let value = parse(SETTINGS).unwrap();
        let text = |key: &str| value.get(key).and_then(Value::as_str);
        assert_eq!(text("target"), Some(r"E:\Archive\Files"));
        assert_eq!(text("Label"), Some(r#"Scratch "disk""#));
        assert_eq!(text("KeepDays"), Some("7"));
        assert_eq!(text("Enabled"), Some("$true"));
        let sets = value.get("Sets").unwrap().items();
        assert_eq!(sets.len(), 3);
        let config = &sets[1];
        let paths: Vec<_> = config
            .get("Paths")
            .unwrap()
            .items()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(paths, ["a.ini", "b.ini", r"tools\c.ini"]);
        assert_eq!(sets[2].get("each").and_then(Value::as_str), Some("WORK"));
        assert_eq!(sets[0].get("Each"), None);
        // One value is a list of one; a comma list is a list; an empty array is empty.
        assert_eq!(value.get("Single").unwrap().items().len(), 1);
        assert_eq!(value.get("Commas").unwrap().items().len(), 2);
        assert!(value.get("Empty").unwrap().items().is_empty());
        assert_eq!(value.get("Missing"), None);
    }

    #[test]
    fn names_the_line_of_a_problem() {
        let cases = [
            ("@{\n  Target = 'x'\n  Broken\n}", "line 3"),
            ("@{\n  Target = 'x\n}", "line 2"),
            ("@{ A = @( 'x' }", "line 1"),
            ("@{ A = 1 } extra", "line 1"),
            ("<# never closed", "line 1"),
            ("@{ A = 1", "not closed"),
        ];
        for (text, expected) in cases {
            let error = parse(text).unwrap_err();
            assert!(error.contains(expected), "{text:?}: {error}");
        }
    }
}
