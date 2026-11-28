//! S-expressions as KiCad writes them (symbol libraries, netlists, boards): nested
//! lists of bare words, numbers and quoted strings.

use openecraft_i18n::tr_fmt;

/// Deepest nesting of lists read: deeper files are refused instead of overflowing the
/// stack when the tree is walked or dropped.
pub const MAX_DEPTH: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub enum Sexpr {
    /// A bare word or number.
    Atom(String),
    /// A quoted string.
    Str(String),
    List(Vec<Sexpr>),
}

impl Sexpr {
    /// The text of a word, number or string; `None` for a list.
    pub fn text(&self) -> Option<&str> {
        match self {
            Sexpr::Atom(s) | Sexpr::Str(s) => Some(s),
            Sexpr::List(_) => None,
        }
    }

    pub fn items(&self) -> &[Sexpr] {
        match self {
            Sexpr::List(v) => v,
            _ => &[],
        }
    }

    /// The keyword of a list: its first item when that is a bare word.
    pub fn name(&self) -> &str {
        match self.items().first() {
            Some(Sexpr::Atom(s)) => s,
            _ => "",
        }
    }

    /// The items after the keyword.
    pub fn args(&self) -> &[Sexpr] {
        self.items().get(1..).unwrap_or(&[])
    }

    /// The first list with the keyword `name` among the items.
    pub fn child(&self, name: &str) -> Option<&Sexpr> {
        self.args().iter().find(|c| c.name() == name)
    }

    pub fn children<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Sexpr> + 'a {
        self.args().iter().filter(move |c| c.name() == name)
    }

    /// The text of argument `i` (0: the first item after the keyword).
    pub fn arg(&self, i: usize) -> Option<&str> {
        self.args().get(i).and_then(Sexpr::text)
    }

    /// Argument `i` as a number.
    pub fn num(&self, i: usize) -> Option<f32> {
        self.arg(i).and_then(|t| t.parse().ok()).filter(|v: &f32| v.is_finite())
    }

    /// The first argument of the child `name`: `(value "10k")` gives `10k`.
    pub fn value(&self, name: &str) -> Option<&str> {
        self.child(name).and_then(|c| c.arg(0))
    }

    /// The bare word `word` is among the arguments (`hide`), or the child of that name
    /// says yes (`(hide yes)`).
    pub fn flag(&self, word: &str) -> bool {
        self.args().iter().any(|a| matches!(a, Sexpr::Atom(s) if s == word)) || self.child(word).is_some_and(|c| c.arg(0) != Some("no"))
    }
}

/// Reads the first expression of `text`.
pub fn parse(text: &str) -> Result<Sexpr, String> {
    let invalid = |line: usize| tr_fmt!("Not a valid KiCad file (line {})", line);
    let mut stack: Vec<Vec<Sexpr>> = Vec::new();
    let mut line = 1;
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(c) = chars.next() {
        let item = match c {
            '\n' => {
                line += 1;
                continue;
            }
            c if c.is_whitespace() => continue,
            '(' => {
                if stack.len() >= MAX_DEPTH {
                    return Err(invalid(line));
                }
                stack.push(Vec::new());
                continue;
            }
            ')' => Sexpr::List(stack.pop().ok_or_else(|| invalid(line))?),
            '"' => {
                let mut s = String::new();
                loop {
                    match chars.next().ok_or_else(|| invalid(line))? {
                        '"' => break,
                        '\\' => match chars.next().ok_or_else(|| invalid(line))? {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => {}
                            other => s.push(other),
                        },
                        '\n' => {
                            line += 1;
                            s.push('\n');
                        }
                        other => s.push(other),
                    }
                }
                Sexpr::Str(s)
            }
            c => {
                let mut s = String::from(c);
                while let Some(&n) = chars.peek() {
                    if n.is_whitespace() || n == '(' || n == ')' || n == '"' {
                        break;
                    }
                    s.push(n);
                    chars.next();
                }
                Sexpr::Atom(s)
            }
        };
        match stack.last_mut() {
            Some(top) => top.push(item),
            None => return Ok(item),
        }
    }
    Err(invalid(line))
}

/// A text as a quoted string.
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A number as KiCad writes it: at most four decimals, no trailing zeros.
pub fn number(v: f32) -> String {
    let v = if v.abs() < 5e-5 { 0.0 } else { v };
    let s = format!("{v:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

#[cfg(test)]
mod tests;
