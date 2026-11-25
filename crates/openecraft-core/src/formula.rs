//! Formulas of project templates: numbers, texts in quotes, values by their name,
//! `+ - * / ^`, comparisons (`= != < <= > >=`, true is 1), `&` (joins as text) and
//! the functions `sqrt`, `abs`, `round(x, digits)`, `ceil`, `floor`, `min`, `max`,
//! `if(condition, then, else)` and `step(x, limit, value, …, otherwise)`: the value
//! of the first limit that `x` does not exceed (a table of sizes, e.g. the setting
//! range of a motor protection switch for a current).

use openecraft_i18n::tr_fmt;
use std::collections::BTreeMap;

/// Result of a formula: a number or a text.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Text(String),
}

impl Value {
    /// A value as given in a template: a number if it reads as one.
    pub fn parse(text: &str) -> Value {
        match text.trim().parse::<f64>() {
            Ok(x) if x.is_finite() => Value::Num(x),
            _ => Value::Text(text.to_string()),
        }
    }

    /// The text a placeholder is replaced with: numbers without trailing zeros.
    pub fn text(&self) -> String {
        match self {
            Value::Text(t) => t.clone(),
            Value::Num(x) => number_text(*x),
        }
    }

    fn truth(&self) -> bool {
        match self {
            Value::Num(x) => *x != 0.0,
            Value::Text(t) => !t.is_empty(),
        }
    }

    fn num(&self) -> Result<f64, String> {
        match self {
            Value::Num(x) => Ok(*x),
            Value::Text(t) => Value::parse(t).as_num().ok_or_else(|| tr_fmt!("{} is not a number", t)),
        }
    }

    fn as_num(&self) -> Option<f64> {
        match self {
            Value::Num(x) => Some(*x),
            Value::Text(_) => None,
        }
    }
}

fn number_text(x: f64) -> String {
    let s = format!("{:.6}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

/// Evaluates `formula` with the values of the names it uses (texts that read as
/// numbers are numbers).
pub fn eval(formula: &str, values: &BTreeMap<String, String>) -> Result<Value, String> {
    let tokens = tokens(formula)?;
    let mut p = Parser { tokens: &tokens, at: 0, values };
    let v = p.compare()?;
    match p.tokens.get(p.at) {
        None => Ok(v),
        Some(t) => Err(tr_fmt!("Syntax error in the formula at {}", t.text())),
    }
}

/// Names a formula uses (values and functions aside).
pub fn names(formula: &str) -> Vec<String> {
    let Ok(tokens) = tokens(formula) else { return Vec::new() };
    let mut v = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        if let Token::Name(n) = t {
            if tokens.get(i + 1) != Some(&Token::Op("(")) && !v.contains(n) {
                v.push(n.clone());
            }
        }
    }
    v
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Num(f64),
    Text(String),
    Name(String),
    Op(&'static str),
}

impl Token {
    fn text(&self) -> String {
        match self {
            Token::Num(x) => number_text(*x),
            Token::Text(t) => format!("\"{t}\""),
            Token::Name(n) => n.clone(),
            Token::Op(o) => o.to_string(),
        }
    }
}

const OPS: [&str; 15] = ["<=", ">=", "!=", "+", "-", "*", "/", "^", "(", ")", ",", "&", "=", "<", ">"];

fn tokens(s: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut v = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            // An exponent: 1e-3.
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let mut j = i + 1;
                if j < chars.len() && (chars[j] == '-' || chars[j] == '+') {
                    j += 1;
                }
                if j < chars.len() && chars[j].is_ascii_digit() {
                    i = j;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let text: String = chars[start..i].iter().collect();
            v.push(Token::Num(text.parse().map_err(|_| tr_fmt!("Syntax error in the formula at {}", text))?));
        } else if c == '"' {
            let end = chars[i + 1..].iter().position(|&d| d == '"').ok_or_else(|| tr_fmt!("Syntax error in the formula at {}", "\""))?;
            v.push(Token::Text(chars[i + 1..i + 1 + end].iter().collect()));
            i += end + 2;
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            v.push(Token::Name(chars[start..i].iter().collect()));
        } else {
            let rest: String = chars[i..chars.len().min(i + 2)].iter().collect();
            let op = OPS.iter().find(|o| rest.starts_with(**o)).ok_or_else(|| tr_fmt!("Syntax error in the formula at {}", c))?;
            v.push(Token::Op(op));
            i += op.chars().count();
        }
    }
    Ok(v)
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
    values: &'a BTreeMap<String, String>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn eat(&mut self, op: &str) -> bool {
        if matches!(self.peek(), Some(Token::Op(o)) if *o == op) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, op: &str) -> Result<(), String> {
        if self.eat(op) {
            Ok(())
        } else {
            Err(self.unexpected())
        }
    }

    fn unexpected(&self) -> String {
        match self.peek() {
            Some(t) => tr_fmt!("Syntax error in the formula at {}", t.text()),
            None => tr_fmt!("The formula ends too early"),
        }
    }

    fn compare(&mut self) -> Result<Value, String> {
        let a = self.join()?;
        for op in ["<=", ">=", "!=", "=", "<", ">"] {
            if self.eat(op) {
                let b = self.join()?;
                let holds = match (a.as_num(), b.as_num()) {
                    (Some(x), Some(y)) => match op {
                        "<=" => x <= y,
                        ">=" => x >= y,
                        "!=" => x != y,
                        "=" => x == y,
                        "<" => x < y,
                        _ => x > y,
                    },
                    _ => match op {
                        "=" => a.text() == b.text(),
                        "!=" => a.text() != b.text(),
                        _ => {
                            let (x, y) = (a.num()?, b.num()?);
                            match op {
                                "<=" => x <= y,
                                ">=" => x >= y,
                                "<" => x < y,
                                _ => x > y,
                            }
                        }
                    },
                };
                return Ok(Value::Num(if holds { 1.0 } else { 0.0 }));
            }
        }
        Ok(a)
    }

    fn join(&mut self) -> Result<Value, String> {
        let mut a = self.sum()?;
        while self.eat("&") {
            let b = self.sum()?;
            a = Value::Text(a.text() + &b.text());
        }
        Ok(a)
    }

    fn sum(&mut self) -> Result<Value, String> {
        let mut a = self.product()?;
        loop {
            if self.eat("+") {
                a = Value::Num(a.num()? + self.product()?.num()?);
            } else if self.eat("-") {
                a = Value::Num(a.num()? - self.product()?.num()?);
            } else {
                return Ok(a);
            }
        }
    }

    fn product(&mut self) -> Result<Value, String> {
        let mut a = self.unary()?;
        loop {
            if self.eat("*") {
                a = Value::Num(a.num()? * self.unary()?.num()?);
            } else if self.eat("/") {
                let b = self.unary()?.num()?;
                if b == 0.0 {
                    return Err(tr_fmt!("Division by zero in the formula"));
                }
                a = Value::Num(a.num()? / b);
            } else {
                return Ok(a);
            }
        }
    }

    fn unary(&mut self) -> Result<Value, String> {
        if self.eat("-") {
            return Ok(Value::Num(-self.unary()?.num()?));
        }
        let a = self.atom()?;
        if self.eat("^") {
            return Ok(Value::Num(a.num()?.powf(self.unary()?.num()?)));
        }
        Ok(a)
    }

    fn atom(&mut self) -> Result<Value, String> {
        let Some(t) = self.peek().cloned() else { return Err(self.unexpected()) };
        self.at += 1;
        match t {
            Token::Num(x) => Ok(Value::Num(x)),
            Token::Text(s) => Ok(Value::Text(s)),
            Token::Op("(") => {
                let v = self.compare()?;
                self.expect(")")?;
                Ok(v)
            }
            Token::Name(n) if self.eat("(") => {
                let mut args = Vec::new();
                if !self.eat(")") {
                    loop {
                        args.push(self.compare()?);
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                call(&n, &args)
            }
            Token::Name(n) => match self.values.get(&n) {
                Some(v) => Ok(Value::parse(v)),
                None => Err(tr_fmt!("Unknown name {} in the formula", n)),
            },
            Token::Op(_) => {
                self.at -= 1;
                Err(self.unexpected())
            }
        }
    }
}

fn call(name: &str, args: &[Value]) -> Result<Value, String> {
    let count = |ok: bool| if ok { Ok(()) } else { Err(tr_fmt!("Wrong number of values for {}", name)) };
    let one = |f: fn(f64) -> f64| -> Result<Value, String> {
        count(args.len() == 1)?;
        Ok(Value::Num(f(args[0].num()?)))
    };
    match name {
        "sqrt" => one(f64::sqrt),
        "abs" => one(f64::abs),
        "ceil" => one(f64::ceil),
        "floor" => one(f64::floor),
        "round" => {
            count(matches!(args.len(), 1 | 2))?;
            let digits = if args.len() == 2 { args[1].num()? } else { 0.0 };
            let f = 10f64.powi(digits as i32);
            Ok(Value::Num((args[0].num()? * f).round() / f))
        }
        "min" | "max" => {
            count(!args.is_empty())?;
            let mut v = args[0].num()?;
            for a in &args[1..] {
                let x = a.num()?;
                v = if name == "min" { v.min(x) } else { v.max(x) };
            }
            Ok(Value::Num(v))
        }
        "if" => {
            count(args.len() == 3)?;
            Ok(if args[0].truth() { args[1].clone() } else { args[2].clone() })
        }
        "step" => {
            count(args.len() >= 3)?;
            let x = args[0].num()?;
            let pairs = &args[1..];
            for p in pairs.chunks(2).filter(|p| p.len() == 2) {
                if x <= p[0].num()? {
                    return Ok(p[1].clone());
                }
            }
            match pairs.len() % 2 {
                1 => Ok(pairs[pairs.len() - 1].clone()),
                _ => Err(tr_fmt!("{} is above the last step", number_text(x))),
            }
        }
        _ => Err(tr_fmt!("Unknown function {} in the formula", name)),
    }
}

#[cfg(test)]
mod tests;
