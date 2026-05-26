//! Restricted condition parser — no raw code injection.

use ir::{BinOp, CmpOp, ValueExpr};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseConditionError {
    #[error("empty condition")]
    Empty,
    #[error("unexpected character '{0}'")]
    UnexpectedChar(char),
    #[error("unclosed parenthesis")]
    UnclosedParen,
    #[error("unexpected end of condition")]
    UnexpectedEof,
    #[error("invalid identifier '{0}'")]
    InvalidIdent(String),
}

/// Parses a small boolean/arithmetic language: literals, idents, == != < <= > >=, && ||, !, ( ).
pub fn parse_condition(input: &str) -> Result<ValueExpr, ParseConditionError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseConditionError::Empty);
    }
    let mut p = Parser::new(s);
    let expr = p.parse_or()?;
    p.skip_ws();
    if !p.rest().is_empty() {
        return Err(ParseConditionError::UnexpectedChar(
            p.rest().chars().next().unwrap_or('?'),
        ));
    }
    Ok(expr)
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    fn rest(&self) -> &str {
        &self.s[self.i..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn parse_or(&mut self) -> Result<ValueExpr, ParseConditionError> {
        let mut left = self.parse_and()?;
        self.skip_ws();
        while self.rest().starts_with("||") {
            self.i += 2;
            self.skip_ws();
            let right = self.parse_and()?;
            left = ValueExpr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
            self.skip_ws();
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<ValueExpr, ParseConditionError> {
        let mut left = self.parse_not()?;
        self.skip_ws();
        while self.rest().starts_with("&&") {
            self.i += 2;
            self.skip_ws();
            let right = self.parse_not()?;
            left = ValueExpr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
            self.skip_ws();
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<ValueExpr, ParseConditionError> {
        self.skip_ws();
        if self.rest().starts_with('!') {
            self.i += 1;
            self.skip_ws();
            let inner = self.parse_not()?;
            return Ok(ValueExpr::Not(Box::new(inner)));
        }
        self.parse_cmp()
    }

    fn parse_cmp(&mut self) -> Result<ValueExpr, ParseConditionError> {
        let left = self.parse_primary()?;
        self.skip_ws();
        let op = if self.rest().starts_with("==") {
            self.i += 2;
            CmpOp::Eq
        } else if self.rest().starts_with("!=") {
            self.i += 2;
            CmpOp::Ne
        } else if self.rest().starts_with("<=") {
            self.i += 2;
            CmpOp::Le
        } else if self.rest().starts_with(">=") {
            self.i += 2;
            CmpOp::Ge
        } else if self.rest().starts_with('<') {
            self.i += 1;
            CmpOp::Lt
        } else if self.rest().starts_with('>') {
            self.i += 1;
            CmpOp::Gt
        } else {
            return Ok(left);
        };
        self.skip_ws();
        let right = self.parse_primary()?;
        Ok(ValueExpr::Cmp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_primary(&mut self) -> Result<ValueExpr, ParseConditionError> {
        self.skip_ws();
        if self.rest().starts_with('(') {
            self.i += 1;
            let inner = self.parse_or()?;
            self.skip_ws();
            if self.peek() != Some(')') {
                return Err(ParseConditionError::UnclosedParen);
            }
            self.bump();
            return Ok(inner);
        }
        if self.rest().starts_with("true") {
            self.i += 4;
            return Ok(ValueExpr::Bool(true));
        }
        if self.rest().starts_with("false") {
            self.i += 5;
            return Ok(ValueExpr::Bool(false));
        }
        if let Some(n) = self.parse_number() {
            return Ok(ValueExpr::I64(n));
        }
        self.parse_ident()
    }

    fn parse_number(&mut self) -> Option<i64> {
        let start = self.i;
        let mut negative = false;
        if self.peek() == Some('-') {
            negative = true;
            self.bump();
        }
        let mut has_digit = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                has_digit = true;
                self.bump();
            } else {
                break;
            }
        }
        if !has_digit {
            self.i = start;
            return None;
        }
        let n: i64 = self.s[start..self.i].trim_start_matches('-').parse().ok()?;
        Some(if negative { -n } else { n })
    }

    fn parse_ident(&mut self) -> Result<ValueExpr, ParseConditionError> {
        let start = self.i;
        if let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() || c == '_' {
                self.bump();
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        self.bump();
                    } else {
                        break;
                    }
                }
            } else {
                return Err(ParseConditionError::UnexpectedChar(c));
            }
        } else {
            return Err(ParseConditionError::UnexpectedEof);
        }
        let name = self.s[start..self.i].to_string();
        if name == "true" || name == "false" {
            return Err(ParseConditionError::InvalidIdent(name));
        }
        Ok(ValueExpr::Ident(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cmp_and_logic() {
        let e = parse_condition("2 > 1 && version == 1").unwrap();
        assert!(matches!(e, ValueExpr::BinOp { op: BinOp::And, .. }));
    }
}
