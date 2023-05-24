use crate::lexer::{Token, TokenKind, Tokenizer};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Peekable;
use thiserror::Error;
use crate::gjson::gjson;
use crate::gjson::gjson::{get_bytes, Kind};

/// Represents the calculated Expression result.
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    String(String),
    Number(f64),
    Bool(bool),
    DateTime(DateTime<Utc>), // What to put here arg! do we preserve the original zone etc..?
    Object(BTreeMap<String, Value>),
    Array(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use serde::ser::Error;

        match serde_json::to_string(self) {
            Ok(s) => {
                f.write_str(&s)?;
                Ok(())
            }
            Err(e) => Err(std::fmt::Error::custom(e)),
        }
    }
}

impl<'a> From<gjson::Value<'a>> for Value {
    fn from(v: gjson::Value) -> Self {
        match v.kind() {
            Kind::Null => Value::Null,
            Kind::String => Value::String(v.str().to_string()),
            Kind::Number => Value::Number(v.f64()),
            Kind::False => Value::Bool(false),
            Kind::True => Value::Bool(true),
            Kind::Array => {
                let arr = v.array().into_iter().map(Into::into).collect();
                Value::Array(arr)
            }
            Kind::Object => {
                let mut m = BTreeMap::new();
                v.each(|k, v| {
                    m.insert(k.str().to_string(), v.into());
                    true
                });
                Value::Object(m)
            }
        }
    }
}

/// Represents a stateless parsed expression that can be applied to JSON data.
pub trait Expression: Debug + Send + Sync {
    /// Will execute the parsed expression and apply it against the supplied json data.
    ///
    /// # Warnings
    ///
    /// This function assumes that the supplied JSON data is valid.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the expression cannot be applied to the supplied data due to invalid
    /// data type comparisons.
    fn calculate(&self, json: &[u8]) -> Result<Value>;
}

/// Is an alias for a Box<dyn Expression>
pub type BoxedExpression = Box<dyn Expression>;

/// Parses a supplied expression and returns a `BoxedExpression`.
pub struct Parser<'a> {
    exp: &'a [u8],
    tokenizer: Peekable<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    fn new(exp: &'a [u8], tokenizer: Peekable<Tokenizer<'a>>) -> Self {
        Parser { exp, tokenizer }
    }

    /// parses the provided expression and turning it into a computation that can be applied to some
    /// source data.
    ///
    /// # Errors
    ///
    /// Will return `Err` the expression is invalid.
    #[inline]
    pub fn parse(expression: &str) -> anyhow::Result<BoxedExpression> {
        Parser::parse_bytes(expression.as_bytes())
    }

    /// parses the provided expression as bytes and turning it into a computation that can be applied to some
    /// source data.
    ///
    /// # Errors
    ///
    /// Will return `Err` the expression is invalid.
    pub fn parse_bytes(expression: &[u8]) -> anyhow::Result<BoxedExpression> {
        let tokenizer = Tokenizer::new_bytes(expression).peekable();
        let mut parser = Parser::new(expression, tokenizer);
        let result = parser.parse_expression()?;

        if let Some(result) = result {
            Ok(result)
        } else {
            Err(anyhow!("no expression results found"))
        }
    }

    #[allow(clippy::too_many_lines)]
    fn parse_expression(&mut self) -> anyhow::Result<Option<BoxedExpression>> {
        let mut current: Option<BoxedExpression> = None;

        loop {
            if let Some(token) = self.tokenizer.next() {
                let token = token?;
                if let Some(expression) = current {
                    // CloseParen is the end of an expression block, return parsed expression.
                    if token.kind == TokenKind::CloseParen {
                        return Ok(Some(expression));
                    }
                    // look for next operation
                    current = self.parse_operation(token, expression)?;
                } else {
                    // look for next value
                    current = Some(self.parse_value(token)?);
                }
            } else {
                return Ok(current);
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn parse_value(&mut self, token: Token) -> anyhow::Result<BoxedExpression> {
        match token.kind {
            TokenKind::OpenBracket => {
                let mut arr = Vec::new();

                loop {
                    if let Some(token) = self.tokenizer.next() {
                        let token = token?;

                        match token.kind {
                            TokenKind::CloseBracket => {
                                break;
                            }
                            TokenKind::Comma => continue, // optional for defining arrays
                            _ => {
                                arr.push(self.parse_value(token)?);
                            }
                        };
                    } else {
                        return Err(anyhow!("unclosed Array '['"));
                    }
                }
                Ok(Box::new(Arr { arr }))
            }
            TokenKind::OpenParen => {
                if let Some(expression) = self.parse_expression()? {
                    Ok(expression)
                } else {
                    Err(anyhow!(
                        "expression after open parenthesis '(' ends unexpectedly."
                    ))
                }
            }
            TokenKind::SelectorPath => {
                let start = token.start as usize;
                Ok(Box::new(SelectorPath {
                    ident: String::from_utf8_lossy(
                        &self.exp[start + 1..(start + token.len as usize)],
                    )
                        .into_owned(),
                }))
            }
            TokenKind::QuotedString => {
                let start = token.start as usize;
                Ok(Box::new(Str {
                    s: String::from_utf8_lossy(
                        &self.exp[start + 1..(start + token.len as usize - 1)],
                    )
                        .into_owned(),
                }))
            }
            TokenKind::Number => {
                let start = token.start as usize;
                Ok(Box::new(Num {
                    n: String::from_utf8_lossy(&self.exp[start..start + token.len as usize])
                        .parse()?,
                }))
            }
            TokenKind::BooleanTrue => Ok(Box::new(Bool { b: true })),
            TokenKind::BooleanFalse => Ok(Box::new(Bool { b: false })),
            TokenKind::Null => Ok(Box::new(Null {})),
            TokenKind::Coerce => {
                // COERCE <expression> _<datatype>_
                let next_token = self.next_operator_token(token)?;
                let const_eligible = matches!(
                    next_token.kind,
                    TokenKind::QuotedString
                        | TokenKind::Number
                        | TokenKind::BooleanFalse
                        | TokenKind::BooleanTrue
                        | TokenKind::Null
                );
                let mut expression = self.parse_value(next_token)?;
                loop {
                    if let Some(token) = self.tokenizer.next() {
                        let token = token?;
                        let start = token.start as usize;

                        if token.kind == TokenKind::Identifier {
                            let ident = String::from_utf8_lossy(
                                &self.exp[start..start + token.len as usize],
                            );
                            match ident.as_ref() {
                                "_datetime_" => {
                                    let value = COERCEDateTime { value: expression };
                                    if const_eligible {
                                        expression = Box::new(CoercedConst {
                                            value: value.calculate(&[])?,
                                        });
                                    } else {
                                        expression = Box::new(value);
                                    }
                                }
                                "_lowercase_" => {
                                    expression = Box::new(CoerceLowercase { value: expression });
                                }
                                "_uppercase_" => {
                                    expression = Box::new(CoerceUppercase { value: expression });
                                }
                                _ => {
                                    return Err(anyhow!("invalid COERCE data type '{:?}'", &ident))
                                }
                            };
                        } else {
                            return Err(anyhow!(
                                "COERCE missing data type identifier, found instead: {:?}",
                                &self.exp[start..(start + token.len as usize)]
                            ));
                        }
                    } else {
                        return Err(anyhow!("no identifier after value for: COERCE"));
                    }
                    if let Some(Ok(token)) = self.tokenizer.peek() {
                        if token.kind == TokenKind::Comma {
                            let _ = self.tokenizer.next(); // consume peeked comma
                            continue;
                        }
                    }
                    break;
                }
                Ok(expression)
            }
            TokenKind::Not => {
                let next_token = self.next_operator_token(token)?;
                let value = self.parse_value(next_token)?;
                Ok(Box::new(Not { value }))
            }
            _ => Err(anyhow!("token is not a valid value: {:?}", token)),
        }
    }

    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    fn next_operator_token(&mut self, operation_token: Token) -> anyhow::Result<Token> {
        if let Some(token) = self.tokenizer.next() {
            Ok(token?)
        } else {
            let start = operation_token.start as usize;
            Err(anyhow!(
                "no value found after operation: {:?}",
                &self.exp[start..(start + operation_token.len as usize)]
            ))
        }
    }

    #[allow(clippy::too_many_lines)]
    fn parse_operation(
        &mut self,
        token: Token,
        current: BoxedExpression,
    ) -> anyhow::Result<Option<BoxedExpression>> {
        match token.kind {
            TokenKind::Add => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Add {
                    left: current,
                    right,
                })))
            }
            TokenKind::Subtract => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Sub {
                    left: current,
                    right,
                })))
            }
            TokenKind::Multiply => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Mult {
                    left: current,
                    right,
                })))
            }
            TokenKind::Divide => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Div {
                    left: current,
                    right,
                })))
            }
            TokenKind::Equals => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Eq {
                    left: current,
                    right,
                })))
            }
            TokenKind::Gt => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Gt {
                    left: current,
                    right,
                })))
            }
            TokenKind::Gte => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Gte {
                    left: current,
                    right,
                })))
            }
            TokenKind::Lt => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Lt {
                    left: current,
                    right,
                })))
            }
            TokenKind::Lte => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Lte {
                    left: current,
                    right,
                })))
            }
            TokenKind::Or => {
                let right = self
                    .parse_expression()?
                    .map_or_else(|| Err(anyhow!("invalid operation after ||")), Ok)?;
                Ok(Some(Box::new(Or {
                    left: current,
                    right,
                })))
            }
            TokenKind::And => {
                let right = self
                    .parse_expression()?
                    .map_or_else(|| Err(anyhow!("invalid operation after &&")), Ok)?;
                Ok(Some(Box::new(And {
                    left: current,
                    right,
                })))
            }
            TokenKind::StartsWith => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(StartsWith {
                    left: current,
                    right,
                })))
            }
            TokenKind::EndsWith => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(EndsWith {
                    left: current,
                    right,
                })))
            }
            TokenKind::In => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(In {
                    left: current,
                    right,
                })))
            }
            TokenKind::Contains => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(Contains {
                    left: current,
                    right,
                })))
            }
            TokenKind::ContainsAny => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(ContainsAny {
                    left: current,
                    right,
                })))
            }
            TokenKind::ContainsAll => {
                let next_token = self.next_operator_token(token)?;
                let right = self.parse_value(next_token)?;
                Ok(Some(Box::new(ContainsAll {
                    left: current,
                    right,
                })))
            }
            TokenKind::Between => {
                let lhs_token = self.next_operator_token(token.clone())?;
                let left = self.parse_value(lhs_token)?;
                let rhs_token = self.next_operator_token(token)?;
                let right = self.parse_value(rhs_token)?;
                Ok(Some(Box::new(Between {
                    left,
                    right,
                    value: current,
                })))
            }
            TokenKind::Not => {
                let next_token = self.next_operator_token(token)?;
                let value = self
                    .parse_operation(next_token, current)?
                    .map_or_else(|| Err(anyhow!("invalid operation after !")), Ok)?;
                Ok(Some(Box::new(Not { value })))
            }
            TokenKind::CloseBracket => Ok(Some(current)),
            _ => Err(anyhow!("invalid operation: {:?}", token)),
        }
    }
}

#[derive(Debug)]
struct Between {
    left: BoxedExpression,
    right: BoxedExpression,
    value: BoxedExpression,
}

impl Expression for Between {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;
        let value = self.value.calculate(json)?;

        match (value, left, right) {
            (Value::String(v), Value::String(lhs), Value::String(rhs)) => {
                Ok(Value::Bool(v > lhs && v < rhs))
            }
            (Value::Number(v), Value::Number(lhs), Value::Number(rhs)) => {
                Ok(Value::Bool(v > lhs && v < rhs))
            }
            (Value::DateTime(v), Value::DateTime(lhs), Value::DateTime(rhs)) => {
                Ok(Value::Bool(v > lhs && v < rhs))
            }
            (Value::Null, _, _) | (_, Value::Null, _) | (_, _, Value::Null) => {
                Ok(Value::Bool(false))
            }
            (v, lhs, rhs) => Err(Error::UnsupportedTypeComparison(format!(
                "{v} BETWEEN {lhs} {rhs}",
            ))),
        }
    }
}

#[derive(Debug)]
struct COERCEDateTime {
    value: BoxedExpression,
}

impl Expression for COERCEDateTime {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let value = self.value.calculate(json)?;

        match value {
            Value::String(ref s) => match anydate::parse_utc(s) {
                Err(_) => Ok(Value::Null),
                Ok(dt) => Ok(Value::DateTime(dt)),
            },
            Value::Null => Ok(value),
            value => Err(Error::UnsupportedCOERCE(
                format!("{value} COERCE datetime",),
            )),
        }
    }
}

#[derive(Debug)]
struct Add {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Add {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(ref s2)) => Ok(Value::String(s1 + s2)),
            (Value::String(s1), Value::Null) => Ok(Value::String(s1)),
            (Value::Null, Value::String(s2)) => Ok(Value::String(s2)),
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
            (Value::Number(n1), Value::Null) => Ok(Value::Number(n1)),
            (Value::Null, Value::Number(n2)) => Ok(Value::Number(n2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} + {r}",))),
        }
    }
}

#[derive(Debug)]
struct Sub {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Sub {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} - {r}",))),
        }
    }
}

#[derive(Debug)]
struct Mult {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Mult {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 * n2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} * {r}",))),
        }
    }
}

#[derive(Debug)]
struct Div {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Div {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 / n2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} / {r}",))),
        }
    }
}

#[derive(Debug)]
struct Eq {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Eq {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;
        Ok(Value::Bool(left == right))
    }
}

#[derive(Debug)]
struct Gt {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Gt {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 > s2)),
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 > n2)),
            (Value::DateTime(dt1), Value::DateTime(dt2)) => Ok(Value::Bool(dt1 > dt2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} > {r}",))),
        }
    }
}

#[derive(Debug)]
struct Gte {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Gte {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 >= s2)),
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 >= n2)),
            (Value::DateTime(dt1), Value::DateTime(dt2)) => Ok(Value::Bool(dt1 >= dt2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} >= {r}",))),
        }
    }
}

#[derive(Debug)]
struct Lt {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Lt {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 < s2)),
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 < n2)),
            (Value::DateTime(dt1), Value::DateTime(dt2)) => Ok(Value::Bool(dt1 < dt2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} < {r}",))),
        }
    }
}

#[derive(Debug)]
struct Lte {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Lte {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 <= s2)),
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 <= n2)),
            (Value::DateTime(dt1), Value::DateTime(dt2)) => Ok(Value::Bool(dt1 <= dt2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} <= {r}",))),
        }
    }
}

#[derive(Debug)]
struct CoercedConst {
    value: Value,
}

impl Expression for CoercedConst {
    fn calculate(&self, _json: &[u8]) -> Result<Value> {
        Ok(self.value.clone())
    }
}

#[derive(Debug)]
struct CoerceLowercase {
    value: BoxedExpression,
}

impl Expression for CoerceLowercase {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let v = self.value.calculate(json)?;
        match v {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            v => Err(Error::UnsupportedCOERCE(format!("{v} COERCE lowercase",))),
        }
    }
}

#[derive(Debug)]
struct CoerceUppercase {
    value: BoxedExpression,
}

impl Expression for CoerceUppercase {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let v = self.value.calculate(json)?;
        match v {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            v => Err(Error::UnsupportedCOERCE(format!("{v} COERCE uppercase",))),
        }
    }
}

#[derive(Debug)]
struct Not {
    value: BoxedExpression,
}

impl Expression for Not {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let v = self.value.calculate(json)?;
        match v {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            v => Err(Error::UnsupportedTypeComparison(format!("{v:?} for !"))),
        }
    }
}

#[derive(Debug)]
struct SelectorPath {
    ident: String,
}

impl Expression for SelectorPath {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        Ok(unsafe { get_bytes(json, &self.ident).into() })
    }
}

#[derive(Debug)]
struct Str {
    s: String,
}

impl Expression for Str {
    fn calculate(&self, _: &[u8]) -> Result<Value> {
        Ok(Value::String(self.s.clone()))
    }
}

#[derive(Debug)]
struct Num {
    n: f64,
}

impl Expression for Num {
    fn calculate(&self, _: &[u8]) -> Result<Value> {
        Ok(Value::Number(self.n))
    }
}

#[derive(Debug)]
struct Bool {
    b: bool,
}

impl Expression for Bool {
    fn calculate(&self, _: &[u8]) -> Result<Value> {
        Ok(Value::Bool(self.b))
    }
}

#[derive(Debug)]
struct Null;

impl Expression for Null {
    fn calculate(&self, _: &[u8]) -> Result<Value> {
        Ok(Value::Null)
    }
}

#[derive(Debug)]
struct Or {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Or {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 || b2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} || {r}",))),
        }
    }
}

#[derive(Debug)]
struct And {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for And {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 && b2)),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} && {r}",))),
        }
    }
}

#[derive(Debug)]
struct Contains {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for Contains {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;
        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1.contains(&s2))),
            (Value::Array(arr1), v) => Ok(Value::Bool(arr1.contains(&v))),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!(
                "{l} CONTAINS {r}",
            ))),
        }
    }
}

#[derive(Debug)]
struct ContainsAny {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for ContainsAny {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;
        match (left, right) {
            (Value::String(s1), Value::String(s2)) => {
                let b1: Vec<char> = s1.chars().collect();
                // betting that lists are short and so less expensive than iterating one to create a hash set
                Ok(Value::Bool(s2.chars().any(|b| b1.contains(&b))))
            }
            (Value::Array(arr1), Value::Array(arr2)) => {
                Ok(Value::Bool(arr2.iter().any(|v| arr1.contains(v))))
            }
            (Value::Array(arr), Value::String(s)) => Ok(Value::Bool(
                s.chars()
                    .any(|v| arr.contains(&Value::String(v.to_string()))),
            )),
            (Value::String(s), Value::Array(arr)) => Ok(Value::Bool(arr.iter().any(|v| match v {
                Value::String(s2) => s.contains(s2),
                _ => false,
            }))),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!(
                "{l} CONTAINS_ANY {r}",
            ))),
        }
    }
}

#[derive(Debug)]
struct ContainsAll {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for ContainsAll {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;
        match (left, right) {
            (Value::String(s1), Value::String(s2)) => {
                let b1: Vec<char> = s1.chars().collect();
                Ok(Value::Bool(s2.chars().all(|b| b1.contains(&b))))
            }
            (Value::Array(arr1), Value::Array(arr2)) => {
                Ok(Value::Bool(arr2.iter().all(|v| arr1.contains(v))))
            }
            (Value::Array(arr), Value::String(s)) => Ok(Value::Bool(
                s.chars()
                    .all(|v| arr.contains(&Value::String(v.to_string()))),
            )),
            (Value::String(s), Value::Array(arr)) => Ok(Value::Bool(arr.iter().all(|v| match v {
                Value::String(s2) => s.contains(s2),
                _ => false,
            }))),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!(
                "{l} CONTAINS_ALL {r}",
            ))),
        }
    }
}

#[derive(Debug)]
struct StartsWith {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for StartsWith {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1.starts_with(&s2))),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} + {r}",))),
        }
    }
}

#[derive(Debug)]
struct EndsWith {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for EndsWith {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1.ends_with(&s2))),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} + {r}",))),
        }
    }
}

#[derive(Debug)]
struct In {
    left: BoxedExpression,
    right: BoxedExpression,
}

impl Expression for In {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let left = self.left.calculate(json)?;
        let right = self.right.calculate(json)?;

        match (left, right) {
            (v, Value::Array(a)) => Ok(Value::Bool(a.contains(&v))),
            (l, r) => Err(Error::UnsupportedTypeComparison(format!("{l} + {r}",))),
        }
    }
}

#[derive(Debug)]
struct Arr {
    arr: Vec<BoxedExpression>,
}

impl Expression for Arr {
    fn calculate(&self, json: &[u8]) -> Result<Value> {
        let mut arr = Vec::new();
        for e in &self.arr {
            arr.push(e.calculate(json)?);
        }
        Ok(Value::Array(arr))
    }
}

/// Result type for the `parse` function.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the expression parser.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("unsupported type comparison: {0}")]
    UnsupportedTypeComparison(String),

    #[error("unsupported COERCE: {0}")]
    UnsupportedCOERCE(String),
}