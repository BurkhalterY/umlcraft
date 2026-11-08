//! A small JavaScript interpreter for the code inside EJS templates (`umlcraft ejs`, src/ejs.rs).
//! StarUML runs templates with real JavaScript in its renderer (src/engine/default-commands.js:1184-1215);
//! UMLCraft runs the subset such templates use, on the model held by the repository, read-only.
//!
//! Supported: `var`/`let`/`const` (with array and object destructuring), `if`/`else`, `for (;;)`,
//! `for…of`, `for…in`, `while`, `do…while`, `switch`, `break`, `continue`, `return`, `throw`,
//! `try`/`catch`/`finally`, function declarations and expressions, arrow functions (default and rest
//! parameters), closures, array and object literals (spread, shorthand, computed keys, methods),
//! template literals, regular expression literals (see src/jsregex.rs), every unary, binary, logical,
//! conditional and assignment operator, `?.`, `??`, `typeof`, `instanceof`, `in`, `delete`, `new Error()`,
//! and the usual methods of strings, arrays, numbers, `Math`, `JSON`, `Object`, `Array`, `Number`,
//! `String`, `parseInt`, `parseFloat`, `isNaN`, `console.log`. Model elements expose their fields
//! (references followed, missing fields at their defaults) and getClassName, getDisplayClassName,
//! getPathname, getNodeText, getParentField, traverse, traverseDepthFirst, traverseField, traverseUp,
//! getTag, getTagValue, getGeneralElements, getSpecialElements, getReturnParameter,
//! getNonReturnParameters, getVisibilityString and getStereotypeString; `type.X` is the class X for
//! `instanceof`; `app.repository` has select, get, getInstancesOf, getRelationshipsOf and getViewsOf, and
//! `app.project` has getProject and getFilename. `_` has sortBy, filter, map, uniq, flatten, compact,
//! includes, isString, isArray, isObject, isNumber, isEmpty, each, find, groupBy, keys, values.
//! Not supported: classes, generators, async code, getters and setters, labels, `with`, `eval`,
//! `require`, prototypes, Date, Map and Set. Writing to model elements is refused.

use crate::model::{Eid, Repo, F};
use indexmap::IndexMap;
use serde_json::Value as Json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ------------------------------------------------------------------------------------------ lexer

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Num(f64),
    Str(String),
    Tpl(Vec<String>, Vec<Vec<Token>>),
    Regex(String, String),
    Id(String),
    P(&'static str),
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub t: Tok,
    /// a line break comes before it
    pub nl: bool,
    pub line: usize,
}

const PUNCTS: &[&str] = &[
    ">>>=", "...", "===", "!==", "**=", "<<=", ">>=", ">>>", "&&=", "||=", "??=", "=>", "==", "!=", "<=", ">=", "&&", "||", "??", "?.", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "**", "<<",
    ">>", "{", "}", "(", ")", "[", "]", ";", ",", "<", ">", "+", "-", "*", "/", "%", "&", "|", "^", "!", "~", "?", ":", "=", ".",
];

struct Lexer<'s> {
    s: &'s [char],
    i: usize,
    line: usize,
}

fn err<T>(m: impl Into<String>) -> Result<T, String> {
    Err(m.into())
}

impl<'s> Lexer<'s> {
    fn peek(&self, k: usize) -> Option<char> {
        self.s.get(self.i + k).copied()
    }

    /// Tokens up to the end, or up to an unmatched `}` when `in_tpl` (the end of a `${…}`).
    fn tokens(&mut self, in_tpl: bool) -> Result<Vec<Token>, String> {
        let mut out: Vec<Token> = vec![];
        let mut depth = 0usize;
        let mut nl = false;
        loop {
            // white space and comments
            while let Some(c) = self.peek(0) {
                if c == '\n' {
                    nl = true;
                    self.line += 1;
                    self.i += 1;
                } else if c.is_whitespace() || c == '\u{feff}' {
                    self.i += 1;
                } else if c == '/' && self.peek(1) == Some('/') {
                    while let Some(c) = self.peek(0) {
                        if c == '\n' {
                            break;
                        }
                        self.i += 1;
                    }
                } else if c == '/' && self.peek(1) == Some('*') {
                    self.i += 2;
                    loop {
                        match self.peek(0) {
                            None => return err("Unterminated comment"),
                            Some('*') if self.peek(1) == Some('/') => {
                                self.i += 2;
                                break;
                            }
                            Some(c) => {
                                if c == '\n' {
                                    nl = true;
                                    self.line += 1;
                                }
                                self.i += 1;
                            }
                        }
                    }
                } else {
                    break;
                }
            }
            let line = self.line;
            let Some(c) = self.peek(0) else {
                if in_tpl {
                    return err("Unterminated template literal");
                }
                out.push(Token { t: Tok::Eof, nl: true, line });
                return Ok(out);
            };
            let t = if c.is_ascii_digit() || (c == '.' && self.peek(1).map(|d| d.is_ascii_digit()).unwrap_or(false)) {
                Tok::Num(self.number()?)
            } else if c == '"' || c == '\'' {
                Tok::Str(self.string(c)?)
            } else if c == '`' {
                self.i += 1;
                self.template()?
            } else if c.is_alphabetic() || c == '_' || c == '$' {
                let st = self.i;
                while let Some(c) = self.peek(0) {
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        self.i += 1;
                    } else {
                        break;
                    }
                }
                Tok::Id(self.s[st..self.i].iter().collect())
            } else if c == '/' && regex_allowed(out.last()) {
                self.regex()?
            } else {
                if in_tpl && c == '}' && depth == 0 {
                    self.i += 1;
                    out.push(Token { t: Tok::Eof, nl: true, line });
                    return Ok(out);
                }
                let mut found = None;
                for p in PUNCTS {
                    let n = p.chars().count();
                    if self.i + n <= self.s.len() && self.s[self.i..self.i + n].iter().copied().eq(p.chars()) {
                        // `?.5` is a conditional and a number
                        if *p == "?." && self.peek(2).map(|d| d.is_ascii_digit()).unwrap_or(false) {
                            continue;
                        }
                        found = Some(*p);
                        break;
                    }
                }
                let Some(p) = found else { return err(format!("Invalid or unexpected token '{c}' (line {})", line + 1)) };
                self.i += p.chars().count();
                if p == "{" {
                    depth += 1;
                } else if p == "}" {
                    depth = depth.saturating_sub(1);
                }
                Tok::P(p)
            };
            out.push(Token { t, nl, line });
            nl = false;
        }
    }

    fn number(&mut self) -> Result<f64, String> {
        let st = self.i;
        if self.peek(0) == Some('0') && matches!(self.peek(1), Some('x' | 'X' | 'b' | 'B' | 'o' | 'O')) {
            let radix = match self.peek(1) {
                Some('x' | 'X') => 16,
                Some('b' | 'B') => 2,
                _ => 8,
            };
            self.i += 2;
            let mut v = 0f64;
            while let Some(d) = self.peek(0).and_then(|c| if c == '_' { Some(u32::MAX) } else { c.to_digit(radix) }) {
                if d != u32::MAX {
                    v = v * radix as f64 + d as f64;
                }
                self.i += 1;
            }
            return Ok(v);
        }
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() || c == '.' || c == '_' {
                self.i += 1;
            } else if (c == 'e' || c == 'E') && self.i > st {
                self.i += 1;
                if matches!(self.peek(0), Some('+' | '-')) {
                    self.i += 1;
                }
            } else {
                break;
            }
        }
        let txt: String = self.s[st..self.i].iter().filter(|c| **c != '_').collect();
        txt.parse::<f64>().map_err(|_| format!("Invalid number {txt}"))
    }

    fn escape(&mut self) -> Result<Option<char>, String> {
        // after the backslash
        let Some(c) = self.peek(0) else { return err("Unterminated string") };
        self.i += 1;
        let hex = |l: &mut Self, n: usize| -> Result<u32, String> {
            let mut v = 0;
            for _ in 0..n {
                let d = l.peek(0).and_then(|c| c.to_digit(16)).ok_or("Invalid escape")?;
                v = v * 16 + d;
                l.i += 1;
            }
            Ok(v)
        };
        Ok(Some(match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'v' => '\u{b}',
            '0' if !self.peek(0).map(|d| d.is_ascii_digit()).unwrap_or(false) => '\0',
            'x' => char::from_u32(hex(self, 2)?).unwrap_or('\u{fffd}'),
            'u' => {
                let v = if self.peek(0) == Some('{') {
                    self.i += 1;
                    let mut v = 0;
                    while let Some(d) = self.peek(0).and_then(|c| c.to_digit(16)) {
                        v = v * 16 + d;
                        self.i += 1;
                    }
                    if self.peek(0) != Some('}') {
                        return err("Invalid Unicode escape");
                    }
                    self.i += 1;
                    v
                } else {
                    let hi = hex(self, 4)?;
                    // a surrogate pair written as two escapes
                    if (0xd800..0xdc00).contains(&hi) && self.peek(0) == Some('\\') && self.peek(1) == Some('u') {
                        self.i += 2;
                        let lo = hex(self, 4)?;
                        0x10000 + ((hi - 0xd800) << 10) + (lo.wrapping_sub(0xdc00) & 0x3ff)
                    } else {
                        hi
                    }
                };
                char::from_u32(v).unwrap_or('\u{fffd}')
            }
            '\r' => {
                if self.peek(0) == Some('\n') {
                    self.i += 1;
                }
                self.line += 1;
                return Ok(None);
            }
            '\n' => {
                self.line += 1;
                return Ok(None);
            }
            c => c,
        }))
    }

    fn string(&mut self, q: char) -> Result<String, String> {
        self.i += 1;
        let mut out = String::new();
        loop {
            match self.peek(0) {
                None | Some('\n') => return err("Unterminated string"),
                Some(c) if c == q => {
                    self.i += 1;
                    return Ok(out);
                }
                Some('\\') => {
                    self.i += 1;
                    if let Some(c) = self.escape()? {
                        out.push(c);
                    }
                }
                Some(c) => {
                    out.push(c);
                    self.i += 1;
                }
            }
        }
    }

    fn template(&mut self) -> Result<Tok, String> {
        let mut quasis = vec![];
        let mut exprs = vec![];
        let mut cur = String::new();
        loop {
            match self.peek(0) {
                None => return err("Unterminated template literal"),
                Some('`') => {
                    self.i += 1;
                    quasis.push(cur);
                    return Ok(Tok::Tpl(quasis, exprs));
                }
                Some('\\') => {
                    self.i += 1;
                    if let Some(c) = self.escape()? {
                        cur.push(c);
                    }
                }
                Some('$') if self.peek(1) == Some('{') => {
                    self.i += 2;
                    quasis.push(std::mem::take(&mut cur));
                    exprs.push(self.tokens(true)?);
                }
                Some(c) => {
                    if c == '\n' {
                        self.line += 1;
                    }
                    cur.push(c);
                    self.i += 1;
                }
            }
        }
    }

    fn regex(&mut self) -> Result<Tok, String> {
        self.i += 1;
        let mut src = String::new();
        let mut class = false;
        loop {
            match self.peek(0) {
                None | Some('\n') => return err("Invalid regular expression: missing /"),
                Some('\\') => {
                    src.push('\\');
                    if let Some(c) = self.peek(1) {
                        src.push(c);
                    }
                    self.i += 2;
                }
                Some('/') if !class => {
                    self.i += 1;
                    break;
                }
                Some(c) => {
                    if c == '[' {
                        class = true;
                    } else if c == ']' {
                        class = false;
                    }
                    src.push(c);
                    self.i += 1;
                }
            }
        }
        let mut flags = String::new();
        while let Some(c) = self.peek(0).filter(|c| c.is_alphabetic()) {
            flags.push(c);
            self.i += 1;
        }
        Ok(Tok::Regex(src, flags))
    }
}

/// Whether a `/` after this token starts a regular expression rather than a division.
fn regex_allowed(prev: Option<&Token>) -> bool {
    match prev.map(|t| &t.t) {
        None => true,
        Some(Tok::Num(_) | Tok::Str(_) | Tok::Tpl(..) | Tok::Regex(..)) => false,
        Some(Tok::Id(k)) => matches!(k.as_str(), "return" | "typeof" | "instanceof" | "in" | "of" | "new" | "delete" | "void" | "throw" | "case" | "do" | "else"),
        Some(Tok::P(p)) => !matches!(*p, ")" | "]" | "}" | "++" | "--"),
        Some(Tok::Eof) => true,
    }
}

pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    Lexer { s: &chars, i: 0, line: 0 }.tokens(false)
}

// ------------------------------------------------------------------------------------------ syntax

#[derive(Debug)]
pub enum Pat {
    Id(String),
    Arr(Vec<Option<Pat>>, Option<Box<Pat>>),
    Obj(Vec<(PKey, Pat)>, Option<String>),
    Def(Box<Pat>, Expr),
    /// an assignment target that is not a name (`a.b = …` in a destructuring assignment)
    Target(Expr),
}

#[derive(Debug)]
pub enum PKey {
    Name(String),
    Computed(Expr),
}

#[derive(Debug)]
pub enum Body {
    Block(Vec<Stmt>),
    Expr(Expr),
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: Option<String>,
    pub params: Vec<Pat>,
    pub rest: Option<Pat>,
    pub body: Body,
    pub arrow: bool,
}

#[derive(Debug)]
pub enum Prop {
    KV(PKey, Expr),
    Spread(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Num(f64),
    Str(Rc<str>),
    Tpl(Vec<Rc<str>>, Vec<Expr>),
    Regex(Rc<str>, Rc<str>),
    Id(String),
    This,
    Arr(Vec<Option<Expr>>),
    Obj(Vec<Prop>),
    Func(Rc<FuncDef>),
    Unary(&'static str, Box<Expr>),
    Update(&'static str, bool, Box<Expr>),
    Bin(&'static str, Box<Expr>, Box<Expr>),
    Logic(&'static str, Box<Expr>, Box<Expr>),
    Assign(&'static str, Box<Pat>, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    /// object, property, `?.` (or after one in the same chain)
    Member(Box<Expr>, Box<Expr>, bool),
    Call(Box<Expr>, Vec<Expr>, bool),
    New(Box<Expr>, Vec<Expr>),
    Spread(Box<Expr>),
    Seq(Vec<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Var(&'static str, Vec<(Pat, Option<Expr>)>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Block(Vec<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    /// declaration kind, target, collection, body, `of` (else `in`)
    ForIn(Option<&'static str>, Pat, Expr, Box<Stmt>, bool),
    While(Expr, Box<Stmt>),
    DoWhile(Box<Stmt>, Expr),
    Break,
    Continue,
    Return(Option<Expr>),
    Throw(Expr),
    Try(Vec<Stmt>, Option<Pat>, Option<Vec<Stmt>>, Option<Vec<Stmt>>),
    Func(String, Rc<FuncDef>),
    Switch(Expr, Vec<(Option<Expr>, Vec<Stmt>)>),
    Empty,
}

struct Parser {
    t: Vec<Token>,
    i: usize,
}

fn bin_prec(p: &str) -> Option<u8> {
    Some(match p {
        "??" => 1,
        "||" => 2,
        "&&" => 3,
        "|" => 4,
        "^" => 5,
        "&" => 6,
        "==" | "!=" | "===" | "!==" => 7,
        "<" | ">" | "<=" | ">=" | "instanceof" | "in" => 8,
        "<<" | ">>" | ">>>" => 9,
        "+" | "-" => 10,
        "*" | "/" | "%" => 11,
        "**" => 12,
        _ => return None,
    })
}

const ASSIGN_OPS: &[&str] = &["=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "&=", "|=", "^=", "&&=", "||=", "??="];

fn static_op(s: &str) -> &'static str {
    match s {
        "instanceof" => "instanceof",
        "in" => "in",
        "typeof" => "typeof",
        "void" => "void",
        "delete" => "delete",
        _ => PUNCTS.iter().find(|p| **p == s).copied().unwrap_or("?"),
    }
}

impl Parser {
    fn cur(&self) -> &Tok {
        &self.t[self.i.min(self.t.len() - 1)].t
    }
    fn at(&self, k: usize) -> &Tok {
        &self.t[(self.i + k).min(self.t.len() - 1)].t
    }
    fn nl(&self) -> bool {
        self.t[self.i.min(self.t.len() - 1)].nl
    }
    fn is(&self, p: &str) -> bool {
        matches!(self.cur(), Tok::P(x) if *x == p)
    }
    fn is_id(&self, k: &str) -> bool {
        matches!(self.cur(), Tok::Id(x) if x == k)
    }
    fn eat(&mut self, p: &str) -> bool {
        if self.is(p) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn eat_id(&mut self, k: &str) -> bool {
        if self.is_id(k) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn unexpected<T>(&self) -> Result<T, String> {
        let line = self.t[self.i.min(self.t.len() - 1)].line + 1;
        match self.cur() {
            Tok::Eof => err("Unexpected end of input"),
            Tok::P(p) => err(format!("Unexpected token '{p}' (line {line})")),
            Tok::Id(x) => err(format!("Unexpected identifier '{x}' (line {line})")),
            Tok::Str(_) | Tok::Tpl(..) => err(format!("Unexpected string (line {line})")),
            Tok::Num(_) => err(format!("Unexpected number (line {line})")),
            Tok::Regex(..) => err(format!("Unexpected regular expression (line {line})")),
        }
    }
    fn expect(&mut self, p: &str) -> Result<(), String> {
        if self.eat(p) {
            Ok(())
        } else {
            self.unexpected()
        }
    }
    fn ident(&mut self) -> Result<String, String> {
        match self.cur().clone() {
            Tok::Id(x) => {
                self.i += 1;
                Ok(x)
            }
            _ => self.unexpected(),
        }
    }
    /// The end of a statement: `;`, or a line break, `}` or the end (automatic semicolons).
    fn semi(&mut self) -> Result<(), String> {
        if self.eat(";") || self.is("}") || matches!(self.cur(), Tok::Eof) || self.nl() {
            Ok(())
        } else {
            self.unexpected()
        }
    }

    fn program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut out = vec![];
        while !matches!(self.cur(), Tok::Eof) {
            out.push(self.stmt()?);
        }
        Ok(out)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect("{")?;
        let mut out = vec![];
        while !self.eat("}") {
            if matches!(self.cur(), Tok::Eof) {
                return self.unexpected();
            }
            out.push(self.stmt()?);
        }
        Ok(out)
    }

    fn stmt(&mut self) -> Result<Stmt, String> {
        if self.is("{") {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.eat(";") {
            return Ok(Stmt::Empty);
        }
        let kw = match self.cur() {
            Tok::Id(k) => k.clone(),
            _ => String::new(),
        };
        match kw.as_str() {
            "var" | "let" | "const" => {
                let d = self.var_decl()?;
                self.semi()?;
                Ok(d)
            }
            "if" => {
                self.i += 1;
                self.expect("(")?;
                let c = self.expr()?;
                self.expect(")")?;
                let a = self.stmt()?;
                let b = if self.eat_id("else") { Some(Box::new(self.stmt()?)) } else { None };
                Ok(Stmt::If(c, Box::new(a), b))
            }
            "for" => self.for_stmt(),
            "while" => {
                self.i += 1;
                self.expect("(")?;
                let c = self.expr()?;
                self.expect(")")?;
                Ok(Stmt::While(c, Box::new(self.stmt()?)))
            }
            "do" => {
                self.i += 1;
                let b = self.stmt()?;
                if !self.eat_id("while") {
                    return self.unexpected();
                }
                self.expect("(")?;
                let c = self.expr()?;
                self.expect(")")?;
                self.eat(";");
                Ok(Stmt::DoWhile(Box::new(b), c))
            }
            "break" | "continue" => {
                self.i += 1;
                self.semi()?;
                Ok(if kw == "break" { Stmt::Break } else { Stmt::Continue })
            }
            "return" => {
                self.i += 1;
                let v = if self.is(";") || self.is("}") || self.nl() || matches!(self.cur(), Tok::Eof) { None } else { Some(self.expr()?) };
                self.semi()?;
                Ok(Stmt::Return(v))
            }
            "throw" => {
                self.i += 1;
                let v = self.expr()?;
                self.semi()?;
                Ok(Stmt::Throw(v))
            }
            "try" => {
                self.i += 1;
                let b = self.block()?;
                let (mut param, mut catch, mut fin) = (None, None, None);
                if self.eat_id("catch") {
                    if self.eat("(") {
                        param = Some(self.pattern()?);
                        self.expect(")")?;
                    }
                    catch = Some(self.block()?);
                }
                if self.eat_id("finally") {
                    fin = Some(self.block()?);
                }
                if catch.is_none() && fin.is_none() {
                    return self.unexpected();
                }
                Ok(Stmt::Try(b, param, catch, fin))
            }
            "function" if matches!(self.at(1), Tok::Id(_)) => {
                self.i += 1;
                let name = self.ident()?;
                let f = self.func_rest(Some(name.clone()))?;
                Ok(Stmt::Func(name, f))
            }
            "switch" => {
                self.i += 1;
                self.expect("(")?;
                let d = self.expr()?;
                self.expect(")")?;
                self.expect("{")?;
                let mut cases = vec![];
                while !self.eat("}") {
                    let test = if self.eat_id("case") {
                        Some(self.expr()?)
                    } else if self.eat_id("default") {
                        None
                    } else {
                        return self.unexpected();
                    };
                    self.expect(":")?;
                    let mut body = vec![];
                    while !self.is("}") && !self.is_id("case") && !self.is_id("default") {
                        if matches!(self.cur(), Tok::Eof) {
                            return self.unexpected();
                        }
                        body.push(self.stmt()?);
                    }
                    cases.push((test, body));
                }
                Ok(Stmt::Switch(d, cases))
            }
            "class" | "async" | "yield" | "await" | "with" | "import" | "export" if !matches!(self.at(1), Tok::P("=" | "." | "(")) => err(format!("UMLCraft templates do not support '{kw}'")),
            _ => {
                let e = self.expr()?;
                self.semi()?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn var_decl(&mut self) -> Result<Stmt, String> {
        let kind: &'static str = match self.ident()?.as_str() {
            "var" => "var",
            "let" => "let",
            _ => "const",
        };
        let mut decls = vec![];
        loop {
            let p = self.pattern()?;
            let init = if self.eat("=") { Some(self.assign()?) } else { None };
            decls.push((p, init));
            if !self.eat(",") {
                break;
            }
        }
        Ok(Stmt::Var(kind, decls))
    }

    fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.i += 1;
        self.expect("(")?;
        // for (x of …), for (const [k, v] of …), for (k in …)
        let save = self.i;
        let kind = match self.cur() {
            Tok::Id(k) if k == "var" || k == "let" || k == "const" => {
                let k: &'static str = if k == "var" { "var" } else if k == "let" { "let" } else { "const" };
                self.i += 1;
                Some(k)
            }
            _ => None,
        };
        if kind.is_some() || matches!(self.cur(), Tok::Id(_)) {
            let p = self.pattern();
            if let Ok(p) = p {
                if self.is_id("of") || self.is_id("in") {
                    let of = self.is_id("of");
                    self.i += 1;
                    let coll = if of { self.assign()? } else { self.expr()? };
                    self.expect(")")?;
                    let body = self.stmt()?;
                    return Ok(Stmt::ForIn(kind, p, coll, Box::new(body), of));
                }
            }
        }
        self.i = save;
        let init = if self.is(";") {
            None
        } else if matches!(self.cur(), Tok::Id(k) if k == "var" || k == "let" || k == "const") {
            Some(Box::new(self.var_decl()?))
        } else {
            Some(Box::new(Stmt::Expr(self.expr()?)))
        };
        self.expect(";")?;
        let test = if self.is(";") { None } else { Some(self.expr()?) };
        self.expect(";")?;
        let upd = if self.is(")") { None } else { Some(self.expr()?) };
        self.expect(")")?;
        let body = self.stmt()?;
        Ok(Stmt::For(init, test, upd, Box::new(body)))
    }

    fn pattern(&mut self) -> Result<Pat, String> {
        let p = if self.eat("[") {
            let mut items = vec![];
            let mut rest = None;
            while !self.eat("]") {
                if self.is(",") {
                    self.i += 1;
                    items.push(None);
                    continue;
                }
                if self.eat("...") {
                    rest = Some(Box::new(self.pattern()?));
                    self.eat(",");
                    continue;
                }
                items.push(Some(self.pattern_def()?));
                if !self.is("]") {
                    self.expect(",")?;
                }
            }
            Pat::Arr(items, rest)
        } else if self.eat("{") {
            let mut items = vec![];
            let mut rest = None;
            while !self.eat("}") {
                if self.eat("...") {
                    rest = Some(self.ident()?);
                    self.eat(",");
                    continue;
                }
                let key = self.prop_key()?;
                let p = if self.eat(":") {
                    self.pattern_def()?
                } else {
                    match &key {
                        PKey::Name(n) => {
                            let p = Pat::Id(n.clone());
                            if self.eat("=") {
                                Pat::Def(Box::new(p), self.assign()?)
                            } else {
                                p
                            }
                        }
                        PKey::Computed(_) => return self.unexpected(),
                    }
                };
                items.push((key, p));
                if !self.is("}") {
                    self.expect(",")?;
                }
            }
            Pat::Obj(items, rest)
        } else {
            Pat::Id(self.ident()?)
        };
        Ok(p)
    }

    /// A pattern with an optional `= default`.
    fn pattern_def(&mut self) -> Result<Pat, String> {
        let p = self.pattern()?;
        Ok(if self.eat("=") { Pat::Def(Box::new(p), self.assign()?) } else { p })
    }

    fn prop_key(&mut self) -> Result<PKey, String> {
        match self.cur().clone() {
            Tok::Id(x) => {
                self.i += 1;
                Ok(PKey::Name(x))
            }
            Tok::Str(s) => {
                self.i += 1;
                Ok(PKey::Name(s))
            }
            Tok::Num(n) => {
                self.i += 1;
                Ok(PKey::Name(num_str(n)))
            }
            Tok::P("[") => {
                self.i += 1;
                let e = self.assign()?;
                self.expect("]")?;
                Ok(PKey::Computed(e))
            }
            _ => self.unexpected(),
        }
    }

    /// Parameters and body, after the name of a function.
    fn func_rest(&mut self, name: Option<String>) -> Result<Rc<FuncDef>, String> {
        self.expect("(")?;
        let (params, rest) = self.params()?;
        let body = Body::Block(self.block()?);
        Ok(Rc::new(FuncDef { name, params, rest, body, arrow: false }))
    }

    /// `a, b = 1, ...c)` up to and including the `)`.
    fn params(&mut self) -> Result<(Vec<Pat>, Option<Pat>), String> {
        let mut ps = vec![];
        let mut rest = None;
        while !self.eat(")") {
            if self.eat("...") {
                rest = Some(self.pattern()?);
                self.eat(",");
                continue;
            }
            let p = self.pattern()?;
            let p = if self.eat("=") { Pat::Def(Box::new(p), self.assign()?) } else { p };
            ps.push(p);
            if !self.is(")") {
                self.expect(",")?;
            }
        }
        Ok((ps, rest))
    }

    fn expr(&mut self) -> Result<Expr, String> {
        let e = self.assign()?;
        if self.is(",") {
            let mut v = vec![e];
            while self.eat(",") {
                v.push(self.assign()?);
            }
            return Ok(Expr::Seq(v));
        }
        Ok(e)
    }

    /// Whether an arrow function starts here.
    fn arrow_ahead(&self) -> bool {
        match self.cur() {
            Tok::Id(k) if k != "function" => matches!(self.at(1), Tok::P("=>")),
            Tok::P("(") => {
                let mut depth = 0;
                let mut k = self.i;
                while k < self.t.len() {
                    match &self.t[k].t {
                        Tok::P("(" | "[" | "{") => depth += 1,
                        Tok::P(")" | "]" | "}") => {
                            depth -= 1;
                            if depth == 0 {
                                return matches!(self.t.get(k + 1).map(|t| &t.t), Some(Tok::P("=>")));
                            }
                        }
                        Tok::Eof => return false,
                        _ => {}
                    }
                    k += 1;
                }
                false
            }
            _ => false,
        }
    }

    fn arrow(&mut self) -> Result<Expr, String> {
        let (params, rest) = if self.eat("(") { self.params()? } else { (vec![Pat::Id(self.ident()?)], None) };
        self.expect("=>")?;
        let body = if self.is("{") { Body::Block(self.block()?) } else { Body::Expr(self.assign()?) };
        Ok(Expr::Func(Rc::new(FuncDef { name: None, params, rest, body, arrow: true })))
    }

    fn assign(&mut self) -> Result<Expr, String> {
        if self.arrow_ahead() {
            return self.arrow();
        }
        if matches!(self.cur(), Tok::Id(k) if k == "async") && (matches!(self.at(1), Tok::Id(k) if k == "function") || matches!(self.at(1), Tok::P("("))) {
            return err("UMLCraft templates do not support async functions");
        }
        let left = self.cond()?;
        if let Tok::P(p) = self.cur().clone() {
            if ASSIGN_OPS.contains(&p) {
                self.i += 1;
                let target = to_pat(left)?;
                let v = self.assign()?;
                return Ok(Expr::Assign(p, Box::new(target), Box::new(v)));
            }
        }
        Ok(left)
    }

    fn cond(&mut self) -> Result<Expr, String> {
        let c = self.binary(1)?;
        if self.eat("?") {
            let a = self.assign()?;
            self.expect(":")?;
            let b = self.assign()?;
            return Ok(Expr::Cond(Box::new(c), Box::new(a), Box::new(b)));
        }
        Ok(c)
    }

    fn binary(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let op = match self.cur() {
                Tok::P(p) => *p,
                Tok::Id(k) if k == "instanceof" || k == "in" => static_op(k),
                _ => break,
            };
            let Some(prec) = bin_prec(op) else { break };
            if prec < min {
                break;
            }
            self.i += 1;
            let right = if op == "**" { self.binary(prec)? } else { self.binary(prec + 1)? };
            left = if matches!(op, "&&" | "||" | "??") { Expr::Logic(op, Box::new(left), Box::new(right)) } else { Expr::Bin(op, Box::new(left), Box::new(right)) };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        let op = match self.cur() {
            Tok::P(p @ ("!" | "-" | "+" | "~")) => Some(*p),
            Tok::Id(k) if matches!(k.as_str(), "typeof" | "void" | "delete") => Some(static_op(k)),
            _ => None,
        };
        if let Some(op) = op {
            self.i += 1;
            let e = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(e)));
        }
        if self.is("++") || self.is("--") {
            let op = if self.is("++") { "++" } else { "--" };
            self.i += 1;
            let e = self.unary()?;
            return Ok(Expr::Update(op, true, Box::new(e)));
        }
        let e = self.call_member()?;
        if (self.is("++") || self.is("--")) && !self.nl() {
            let op = if self.is("++") { "++" } else { "--" };
            self.i += 1;
            return Ok(Expr::Update(op, false, Box::new(e)));
        }
        Ok(e)
    }

    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut out = vec![];
        while !self.eat(")") {
            if self.eat("...") {
                out.push(Expr::Spread(Box::new(self.assign()?)));
            } else {
                out.push(self.assign()?);
            }
            if !self.is(")") {
                self.expect(",")?;
            }
        }
        Ok(out)
    }

    fn call_member(&mut self) -> Result<Expr, String> {
        let mut e = if self.eat_id("new") {
            let mut callee = self.primary()?;
            while self.eat(".") {
                let n = self.ident()?;
                callee = Expr::Member(Box::new(callee), Box::new(Expr::Str(n.into())), false);
            }
            let args = if self.eat("(") { self.args()? } else { vec![] };
            Expr::New(Box::new(callee), args)
        } else {
            self.primary()?
        };
        let mut chain = false;
        loop {
            if self.eat(".") {
                let n = self.ident()?;
                e = Expr::Member(Box::new(e), Box::new(Expr::Str(n.into())), chain);
            } else if self.eat("?.") {
                chain = true;
                if self.eat("(") {
                    e = Expr::Call(Box::new(e), self.args()?, true);
                } else if self.eat("[") {
                    let k = self.expr()?;
                    self.expect("]")?;
                    e = Expr::Member(Box::new(e), Box::new(k), true);
                } else {
                    let n = self.ident()?;
                    e = Expr::Member(Box::new(e), Box::new(Expr::Str(n.into())), true);
                }
            } else if self.is("[") {
                self.i += 1;
                let k = self.expr()?;
                self.expect("]")?;
                e = Expr::Member(Box::new(e), Box::new(k), chain);
            } else if self.is("(") {
                self.i += 1;
                e = Expr::Call(Box::new(e), self.args()?, chain);
            } else if matches!(self.cur(), Tok::Tpl(..)) && !self.nl() {
                return err("UMLCraft templates do not support tagged templates");
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let t = self.cur().clone();
        self.i += 1;
        Ok(match t {
            Tok::Num(n) => Expr::Num(n),
            Tok::Str(s) => Expr::Str(s.into()),
            Tok::Regex(s, f) => Expr::Regex(s.into(), f.into()),
            Tok::Tpl(q, es) => {
                let mut exprs = vec![];
                for toks in es {
                    let mut p = Parser { t: toks, i: 0 };
                    let e = p.expr()?;
                    if !matches!(p.cur(), Tok::Eof) {
                        return p.unexpected();
                    }
                    exprs.push(e);
                }
                Expr::Tpl(q.into_iter().map(|s| s.into()).collect(), exprs)
            }
            Tok::Id(k) => match k.as_str() {
                "this" => Expr::This,
                "true" => Expr::Id("true".into()),
                "function" => {
                    let name = if let Tok::Id(n) = self.cur().clone() {
                        self.i += 1;
                        Some(n)
                    } else {
                        None
                    };
                    Expr::Func(self.func_rest(name)?)
                }
                "class" => return err("UMLCraft templates do not support classes"),
                _ => Expr::Id(k),
            },
            Tok::P("(") => {
                let e = self.expr()?;
                self.expect(")")?;
                e
            }
            Tok::P("[") => {
                let mut items = vec![];
                while !self.eat("]") {
                    if self.is(",") {
                        self.i += 1;
                        items.push(None);
                        continue;
                    }
                    if self.eat("...") {
                        items.push(Some(Expr::Spread(Box::new(self.assign()?))));
                    } else {
                        items.push(Some(self.assign()?));
                    }
                    if !self.is("]") {
                        self.expect(",")?;
                    }
                }
                Expr::Arr(items)
            }
            Tok::P("{") => {
                let mut props = vec![];
                while !self.eat("}") {
                    if self.eat("...") {
                        props.push(Prop::Spread(self.assign()?));
                    } else {
                        let key = self.prop_key()?;
                        if self.eat(":") {
                            props.push(Prop::KV(key, self.assign()?));
                        } else if self.is("(") {
                            let name = match &key {
                                PKey::Name(n) => Some(n.clone()),
                                _ => None,
                            };
                            props.push(Prop::KV(key, Expr::Func(self.func_rest(name)?)));
                        } else {
                            match key {
                                PKey::Name(n) => props.push(Prop::KV(PKey::Name(n.clone()), Expr::Id(n))),
                                PKey::Computed(_) => return self.unexpected(),
                            }
                        }
                    }
                    if !self.is("}") {
                        self.expect(",")?;
                    }
                }
                Expr::Obj(props)
            }
            _ => {
                self.i -= 1;
                return self.unexpected();
            }
        })
    }
}

/// An expression on the left of `=` as an assignment target.
fn to_pat(e: Expr) -> Result<Pat, String> {
    Ok(match e {
        Expr::Id(n) => Pat::Id(n),
        Expr::Member(..) => Pat::Target(e),
        Expr::Arr(items) => {
            let mut out = vec![];
            let mut rest = None;
            for it in items {
                match it {
                    Some(Expr::Spread(x)) => rest = Some(Box::new(to_pat(*x)?)),
                    Some(x) => out.push(Some(to_pat(x)?)),
                    None => out.push(None),
                }
            }
            Pat::Arr(out, rest)
        }
        Expr::Obj(props) => {
            let mut out = vec![];
            for p in props {
                match p {
                    Prop::KV(k, v) => out.push((k, to_pat(v)?)),
                    Prop::Spread(_) => return err("Invalid destructuring assignment target"),
                }
            }
            Pat::Obj(out, None)
        }
        Expr::Assign("=", p, d) => Pat::Def(p, *d),
        _ => return err("Invalid left-hand side in assignment"),
    })
}

pub fn parse(src: &str) -> Result<Vec<Stmt>, String> {
    let t = lex(src)?;
    Parser { t, i: 0 }.program()
}

// ------------------------------------------------------------------------------------------ values

pub type Arr = Rc<RefCell<Vec<V>>>;
pub type Obj = Rc<RefCell<IndexMap<String, V>>>;

#[derive(Clone)]
pub enum V {
    Undef,
    Null,
    Bool(bool),
    Num(f64),
    Str(Rc<str>),
    Arr(Arr),
    Obj(Obj),
    Func(Rc<Func>),
    /// a model element
    Elem(Eid),
    /// `type.X`
    Type(Rc<str>),
    /// a built-in object such as Math, app.repository or `_`
    Host(&'static str),
    Regex(Rc<crate::jsregex::Regex>),
}

pub enum Func {
    Closure(Rc<FuncDef>, Env, Option<V>),
    /// a built-in method bound to its object
    Method(V, Rc<str>),
    /// a built-in function by name
    Native(&'static str),
}

pub fn s(x: &str) -> V {
    V::Str(x.into())
}

pub fn arr(v: Vec<V>) -> V {
    V::Arr(Rc::new(RefCell::new(v)))
}

pub fn obj(v: IndexMap<String, V>) -> V {
    V::Obj(Rc::new(RefCell::new(v)))
}

fn native(n: &'static str) -> V {
    V::Func(Rc::new(Func::Native(n)))
}

fn method(this: &V, n: &str) -> V {
    V::Func(Rc::new(Func::Method(this.clone(), n.into())))
}

/// How JavaScript writes a number.
pub fn num_str(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity".into() } else { "-Infinity".into() }
    } else if n == 0.0 {
        "0".into()
    } else if n.fract() == 0.0 && n.abs() < 1e21 {
        format!("{n:.0}")
    } else if n.abs() >= 1e21 || n.abs() < 1e-6 {
        let t = format!("{n:e}");
        // 1e21 → "1e+21", as JavaScript writes it
        match t.split_once('e') {
            Some((m, e)) if !e.starts_with('-') => format!("{m}e+{e}"),
            _ => t,
        }
    } else {
        format!("{n}")
    }
}

// ------------------------------------------------------------------------------------------ scopes

pub type Env = Rc<RefCell<Scope>>;

pub struct Scope {
    vars: HashMap<String, V>,
    parent: Option<Env>,
    /// a function's (or the template's) scope, where `var` declares
    func: bool,
}

fn scope(parent: Option<Env>, func: bool) -> Env {
    Rc::new(RefCell::new(Scope { vars: HashMap::new(), parent, func }))
}

fn lookup(env: &Env, name: &str) -> Option<V> {
    let mut cur = Some(env.clone());
    while let Some(e) = cur {
        if let Some(v) = e.borrow().vars.get(name) {
            return Some(v.clone());
        }
        cur = e.borrow().parent.clone();
    }
    None
}

/// Assign to an existing variable; false if there is none.
fn store(env: &Env, name: &str, v: V) -> bool {
    let mut cur = Some(env.clone());
    while let Some(e) = cur {
        if let Some(slot) = e.borrow_mut().vars.get_mut(name) {
            *slot = v;
            return true;
        }
        cur = e.borrow().parent.clone();
    }
    false
}

fn declare(env: &Env, name: &str, v: V) {
    env.borrow_mut().vars.insert(name.to_string(), v);
}

pub fn declare_in(env: &Env, name: &str, v: V) {
    declare(env, name, v);
}

fn func_scope(env: &Env) -> Env {
    let mut cur = env.clone();
    loop {
        if cur.borrow().func {
            return cur;
        }
        let p = cur.borrow().parent.clone();
        match p {
            Some(p) => cur = p,
            None => return cur,
        }
    }
}

fn root_scope(env: &Env) -> Env {
    let mut cur = env.clone();
    loop {
        let p = cur.borrow().parent.clone();
        match p {
            Some(p) => cur = p,
            None => return cur,
        }
    }
}

// ------------------------------------------------------------------------------------------ evaluation

enum Place {
    Var(String),
    Prop(V, V),
}

enum Ctl {
    Norm,
    Brk,
    Cont,
    Ret(V),
}

/// A thrown value.
pub struct Thrown(pub V);
type R<T> = Result<T, Thrown>;

fn error_obj(name: &str, msg: &str) -> V {
    let mut m = IndexMap::new();
    m.insert("name".to_string(), s(name));
    m.insert("message".to_string(), s(msg));
    obj(m)
}

fn type_err<T>(msg: impl AsRef<str>) -> R<T> {
    Err(Thrown(error_obj("TypeError", msg.as_ref())))
}

fn throw<T>(msg: impl AsRef<str>) -> R<T> {
    Err(Thrown(error_obj("Error", msg.as_ref())))
}

/// The global functions and objects every template sees.
const GLOBALS: &[(&str, &str)] = &[
    ("Math", "host"), ("JSON", "host"), ("Object", "host"), ("Array", "host"), ("Number", "host"), ("String", "host"), ("Boolean", "host"), ("console", "host"), ("app", "host"), ("type", "host"), ("_", "host"),
    ("parseInt", "fn"), ("parseFloat", "fn"), ("isNaN", "fn"), ("isFinite", "fn"), ("filenamify", "fn"), ("Error", "fn"), ("TypeError", "fn"), ("encodeURIComponent", "fn"), ("decodeURIComponent", "fn"),
    ("__append", "fn"), ("__escape", "fn"), ("include", "fn"),
];

pub struct Interp<'a> {
    pub repo: &'a Repo,
    /// the project's file
    pub filename: String,
    pub globals: Env,
    steps: u64,
    /// the output of the template being rendered (and of the templates it includes)
    pub out: Vec<String>,
    /// renders an `include(path, data)`: the hook gets the interpreter, the path and the data
    pub include: Option<fn(&mut Interp, &str, V) -> R<V>>,
    /// the folders of the templates being rendered, innermost last
    pub dirs: Vec<std::path::PathBuf>,
}

const MAX_STEPS: u64 = 200_000_000;

impl<'a> Interp<'a> {
    pub fn new(repo: &'a Repo, filename: &str) -> Interp<'a> {
        let g = scope(None, true);
        for (n, k) in GLOBALS {
            let v = if *k == "host" { V::Host(n) } else { native(n) };
            declare(&g, n, v);
        }
        declare(&g, "undefined", V::Undef);
        declare(&g, "NaN", V::Num(f64::NAN));
        declare(&g, "Infinity", V::Num(f64::INFINITY));
        Interp { repo, filename: filename.to_string(), globals: g, steps: 0, out: vec![String::new()], include: None, dirs: vec![] }
    }

    pub fn set_global(&mut self, name: &str, v: V) {
        declare(&self.globals, name, v);
    }

    /// Run a program in a scope of the globals.
    pub fn run(&mut self, prog: &[Stmt], env: &Env) -> R<()> {
        self.hoist(prog, env);
        match self.block(prog, env)? {
            Ctl::Norm | Ctl::Ret(_) => Ok(()),
            Ctl::Brk | Ctl::Cont => throw("Illegal break or continue statement"),
        }
    }

    pub fn new_scope(&self) -> Env {
        scope(Some(self.globals.clone()), true)
    }

    fn tick(&mut self) -> R<()> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return throw("The template runs too long (an endless loop?)");
        }
        Ok(())
    }

    /// Function declarations are known in their whole block.
    fn hoist(&mut self, stmts: &[Stmt], env: &Env) {
        for st in stmts {
            if let Stmt::Func(n, f) = st {
                declare(env, n, V::Func(Rc::new(Func::Closure(f.clone(), env.clone(), None))));
            }
        }
    }

    fn block(&mut self, stmts: &[Stmt], env: &Env) -> R<Ctl> {
        for st in stmts {
            match self.stmt(st, env)? {
                Ctl::Norm => {}
                c => return Ok(c),
            }
        }
        Ok(Ctl::Norm)
    }

    fn sub_block(&mut self, stmts: &[Stmt], env: &Env) -> R<Ctl> {
        let e = scope(Some(env.clone()), false);
        self.hoist(stmts, &e);
        self.block(stmts, &e)
    }

    fn stmt(&mut self, st: &Stmt, env: &Env) -> R<Ctl> {
        self.tick()?;
        match st {
            Stmt::Expr(e) => {
                self.eval(e, env)?;
            }
            Stmt::Var(kind, decls) => {
                for (p, init) in decls {
                    let v = match init {
                        Some(e) => self.eval(e, env)?,
                        None => {
                            // `var x;` keeps an existing value
                            if let (Pat::Id(n), true) = (p, *kind == "var") {
                                if func_scope(env).borrow().vars.contains_key(n) {
                                    continue;
                                }
                            }
                            V::Undef
                        }
                    };
                    let target = if *kind == "var" { func_scope(env) } else { env.clone() };
                    self.bind(p, v, &target, true)?;
                }
            }
            Stmt::If(c, a, b) => {
                let c = self.eval(c, env)?;
                if truthy(&c) {
                    return self.stmt_scoped(a, env);
                } else if let Some(b) = b {
                    return self.stmt_scoped(b, env);
                }
            }
            Stmt::Block(b) => return self.sub_block(b, env),
            Stmt::For(init, test, upd, body) => {
                let e = scope(Some(env.clone()), false);
                if let Some(i) = init {
                    self.stmt(i, &e)?;
                }
                loop {
                    self.tick()?;
                    if let Some(t) = test {
                        if !truthy(&self.eval(t, &e)?) {
                            break;
                        }
                    }
                    match self.stmt_scoped(body, &e)? {
                        Ctl::Brk => break,
                        Ctl::Ret(v) => return Ok(Ctl::Ret(v)),
                        _ => {}
                    }
                    if let Some(u) = upd {
                        self.eval(u, &e)?;
                    }
                }
            }
            Stmt::ForIn(kind, p, coll, body, of) => {
                let c = self.eval(coll, env)?;
                let items: Vec<V> = if *of {
                    self.iterate(&c)?
                } else {
                    self.keys(&c).into_iter().map(|k| s(&k)).collect()
                };
                for it in items {
                    self.tick()?;
                    let e = scope(Some(env.clone()), false);
                    match kind {
                        Some("var") => self.bind(p, it, &func_scope(env), true)?,
                        Some(_) => self.bind(p, it, &e, true)?,
                        None => self.bind(p, it, env, false)?,
                    }
                    match self.stmt_scoped(body, &e)? {
                        Ctl::Brk => break,
                        Ctl::Ret(v) => return Ok(Ctl::Ret(v)),
                        _ => {}
                    }
                }
            }
            Stmt::While(c, body) => loop {
                self.tick()?;
                if !truthy(&self.eval(c, env)?) {
                    break;
                }
                match self.stmt_scoped(body, env)? {
                    Ctl::Brk => break,
                    Ctl::Ret(v) => return Ok(Ctl::Ret(v)),
                    _ => {}
                }
            },
            Stmt::DoWhile(body, c) => loop {
                self.tick()?;
                match self.stmt_scoped(body, env)? {
                    Ctl::Brk => break,
                    Ctl::Ret(v) => return Ok(Ctl::Ret(v)),
                    _ => {}
                }
                if !truthy(&self.eval(c, env)?) {
                    break;
                }
            },
            Stmt::Break => return Ok(Ctl::Brk),
            Stmt::Continue => return Ok(Ctl::Cont),
            Stmt::Return(e) => {
                let v = match e {
                    Some(e) => self.eval(e, env)?,
                    None => V::Undef,
                };
                return Ok(Ctl::Ret(v));
            }
            Stmt::Throw(e) => {
                let v = self.eval(e, env)?;
                return Err(Thrown(v));
            }
            Stmt::Try(b, param, catch, fin) => {
                let r = self.sub_block(b, env);
                let r = match (r, catch) {
                    (Err(Thrown(v)), Some(c)) => {
                        let e = scope(Some(env.clone()), false);
                        if let Some(p) = param {
                            self.bind(p, v, &e, true)?;
                        }
                        self.hoist(c, &e);
                        self.block(c, &e)
                    }
                    (r, _) => r,
                };
                if let Some(f) = fin {
                    match self.sub_block(f, env)? {
                        Ctl::Norm => {}
                        c => return Ok(c),
                    }
                }
                return r;
            }
            Stmt::Func(..) => {}
            Stmt::Switch(d, cases) => {
                let d = self.eval(d, env)?;
                let e = scope(Some(env.clone()), false);
                let mut start = None;
                for (i, (t, _)) in cases.iter().enumerate() {
                    if let Some(t) = t {
                        let v = self.eval(t, &e)?;
                        if strict_eq(&d, &v) {
                            start = Some(i);
                            break;
                        }
                    }
                }
                let start = start.or_else(|| cases.iter().position(|(t, _)| t.is_none()));
                if let Some(st) = start {
                    for (_, body) in &cases[st..] {
                        match self.block(body, &e)? {
                            Ctl::Norm => {}
                            Ctl::Brk => break,
                            c => return Ok(c),
                        }
                    }
                }
            }
            Stmt::Empty => {}
        }
        Ok(Ctl::Norm)
    }

    /// A statement that is the body of an if or a loop (a block gets its own scope).
    fn stmt_scoped(&mut self, st: &Stmt, env: &Env) -> R<Ctl> {
        match st {
            Stmt::Block(b) => self.sub_block(b, env),
            _ => self.stmt(st, env),
        }
    }

    /// Bind (declare) or assign a pattern.
    fn bind(&mut self, p: &Pat, v: V, env: &Env, decl: bool) -> R<()> {
        match p {
            Pat::Id(n) => {
                if decl {
                    declare(env, n, v);
                } else if !store(env, n, v.clone()) {
                    // an undeclared name becomes a global, as in sloppy JavaScript
                    declare(&root_scope(env), n, v);
                }
            }
            Pat::Def(p, d) => {
                let v = if matches!(v, V::Undef) { self.eval(d, env)? } else { v };
                self.bind(p, v, env, decl)?;
            }
            Pat::Arr(items, rest) => {
                let all = self.iterate(&v)?;
                for (i, it) in items.iter().enumerate() {
                    if let Some(p) = it {
                        self.bind(p, all.get(i).cloned().unwrap_or(V::Undef), env, decl)?;
                    }
                }
                if let Some(r) = rest {
                    let tail = all.get(items.len()..).map(|t| t.to_vec()).unwrap_or_default();
                    self.bind(r, arr(tail), env, decl)?;
                }
            }
            Pat::Obj(items, rest) => {
                if matches!(v, V::Undef | V::Null) {
                    return type_err(format!("Cannot destructure '{}' as it is {}.", to_str(&v), to_str(&v)));
                }
                let mut used = vec![];
                for (k, p) in items {
                    let k = match k {
                        PKey::Name(n) => n.clone(),
                        PKey::Computed(e) => {
                            let k = self.eval(e, env)?;
                            to_str(&k)
                        }
                    };
                    let x = self.get(&v, &k)?;
                    used.push(k);
                    self.bind(p, x, env, decl)?;
                }
                if let Some(r) = rest {
                    let mut m = IndexMap::new();
                    for k in self.keys(&v) {
                        if !used.contains(&k) {
                            let x = self.get(&v, &k)?;
                            m.insert(k, x);
                        }
                    }
                    self.bind(&Pat::Id(r.clone()), obj(m), env, decl)?;
                }
            }
            Pat::Target(e) => {
                let Expr::Member(o, k, _) = e else { return throw("Invalid assignment target") };
                let o = self.eval(o, env)?;
                let k = self.eval(k, env)?;
                self.put(&o, &k, v)?;
            }
        }
        Ok(())
    }

    /// The items of an iterable (array, string, or an object's values for `_` helpers).
    pub fn iterate(&mut self, v: &V) -> R<Vec<V>> {
        Ok(match v {
            V::Arr(a) => a.borrow().clone(),
            V::Str(x) => x.chars().map(|c| s(&c.to_string())).collect(),
            _ => return type_err(format!("{} is not iterable", to_str(v))),
        })
    }

    /// The enumerable keys, as `for…in` and Object.keys see them.
    pub fn keys(&self, v: &V) -> Vec<String> {
        match v {
            V::Arr(a) => (0..a.borrow().len()).map(|i| i.to_string()).collect(),
            V::Obj(o) => o.borrow().keys().cloned().collect(),
            V::Str(x) => (0..x.chars().count()).map(|i| i.to_string()).collect(),
            V::Elem(e) => {
                let mut out = vec!["_id".to_string(), "_parent".to_string()];
                for (k, _) in &self.repo.e(*e).fields {
                    if !out.contains(k) {
                        out.push(k.clone());
                    }
                }
                out
            }
            _ => vec![],
        }
    }

    pub fn eval(&mut self, e: &Expr, env: &Env) -> R<V> {
        Ok(match e {
            Expr::Num(n) => V::Num(*n),
            Expr::Str(x) => V::Str(x.clone()),
            Expr::Tpl(q, es) => {
                let mut out = String::new();
                for (i, part) in q.iter().enumerate() {
                    out.push_str(part);
                    if let Some(e) = es.get(i) {
                        let v = self.eval(e, env)?;
                        out.push_str(&self.string(&v)?);
                    }
                }
                s(&out)
            }
            Expr::Regex(src, flags) => match crate::jsregex::Regex::new(src, flags) {
                Ok(r) => V::Regex(Rc::new(r)),
                Err(m) => return Err(Thrown(error_obj("SyntaxError", &format!("Invalid regular expression: /{src}/: {m}")))),
            },
            Expr::Id(n) => match n.as_str() {
                "true" => V::Bool(true),
                "false" => V::Bool(false),
                "null" => V::Null,
                _ => match lookup(env, n) {
                    Some(v) => v,
                    None => return Err(Thrown(error_obj("ReferenceError", &format!("{n} is not defined")))),
                },
            },
            Expr::This => lookup(env, "this").unwrap_or(V::Undef),
            Expr::Arr(items) => {
                let mut out = vec![];
                for it in items {
                    match it {
                        Some(Expr::Spread(x)) => {
                            let v = self.eval(x, env)?;
                            out.extend(self.iterate(&v)?);
                        }
                        Some(x) => out.push(self.eval(x, env)?),
                        None => out.push(V::Undef),
                    }
                }
                arr(out)
            }
            Expr::Obj(props) => {
                let mut m = IndexMap::new();
                for p in props {
                    match p {
                        Prop::KV(k, v) => {
                            let k = match k {
                                PKey::Name(n) => n.clone(),
                                PKey::Computed(e) => {
                                    let k = self.eval(e, env)?;
                                    to_str(&k)
                                }
                            };
                            let v = self.eval(v, env)?;
                            m.insert(k, v);
                        }
                        Prop::Spread(x) => {
                            let v = self.eval(x, env)?;
                            for k in self.keys(&v) {
                                let x = self.get(&v, &k)?;
                                m.insert(k, x);
                            }
                        }
                    }
                }
                obj(m)
            }
            Expr::Func(f) => {
                let this = if f.arrow { Some(lookup(env, "this").unwrap_or(V::Undef)) } else { None };
                V::Func(Rc::new(Func::Closure(f.clone(), env.clone(), this)))
            }
            Expr::Unary(op, x) => {
                if *op == "typeof" {
                    if let Expr::Id(n) = &**x {
                        if lookup(env, n).is_none() && !matches!(n.as_str(), "true" | "false" | "null") {
                            return Ok(s("undefined"));
                        }
                    }
                    let v = self.eval(x, env)?;
                    return Ok(s(type_of(&v)));
                }
                if *op == "delete" {
                    if let Expr::Member(o, k, _) = &**x {
                        let o = self.eval(o, env)?;
                        let k = self.eval(k, env)?;
                        match o {
                            V::Obj(m) => {
                                m.borrow_mut().shift_remove(&to_str(&k));
                            }
                            V::Elem(_) => return type_err("Model elements are read-only in UMLCraft templates"),
                            _ => {}
                        }
                    }
                    return Ok(V::Bool(true));
                }
                let v = self.eval(x, env)?;
                match *op {
                    "!" => V::Bool(!truthy(&v)),
                    "-" => V::Num(-self.number(&v)?),
                    "+" => V::Num(self.number(&v)?),
                    "~" => V::Num(!(to_i32(self.number(&v)?)) as f64),
                    _ => V::Undef,
                }
            }
            Expr::Update(op, prefix, target) => {
                let place = self.place(target, env)?;
                let old = self.read_place(&place, env)?;
                let n = self.number(&old)?;
                let new = if *op == "++" { n + 1.0 } else { n - 1.0 };
                self.write_place(&place, V::Num(new), env)?;
                V::Num(if *prefix { new } else { n })
            }
            Expr::Bin(op, a, b) => {
                let a = self.eval(a, env)?;
                let b = self.eval(b, env)?;
                self.binop(op, &a, &b)?
            }
            Expr::Logic(op, a, b) => {
                let a = self.eval(a, env)?;
                match *op {
                    "&&" => {
                        if truthy(&a) {
                            self.eval(b, env)?
                        } else {
                            a
                        }
                    }
                    "||" => {
                        if truthy(&a) {
                            a
                        } else {
                            self.eval(b, env)?
                        }
                    }
                    _ => {
                        if matches!(a, V::Undef | V::Null) {
                            self.eval(b, env)?
                        } else {
                            a
                        }
                    }
                }
            }
            Expr::Assign(op, p, v) => {
                if *op == "=" {
                    let v = self.eval(v, env)?;
                    self.bind(p, v.clone(), env, false)?;
                    return Ok(v);
                }
                let place = match &**p {
                    Pat::Id(n) => Place::Var(n.clone()),
                    Pat::Target(t) => self.place(t, env)?,
                    _ => return throw("Invalid left-hand side in assignment"),
                };
                let cur = self.read_place(&place, env)?;
                let nv = match *op {
                    "&&=" => {
                        if !truthy(&cur) {
                            return Ok(cur);
                        }
                        self.eval(v, env)?
                    }
                    "||=" => {
                        if truthy(&cur) {
                            return Ok(cur);
                        }
                        self.eval(v, env)?
                    }
                    "??=" => {
                        if !matches!(cur, V::Undef | V::Null) {
                            return Ok(cur);
                        }
                        self.eval(v, env)?
                    }
                    _ => {
                        let r = self.eval(v, env)?;
                        self.binop(&op[..op.len() - 1], &cur, &r)?
                    }
                };
                self.write_place(&place, nv.clone(), env)?;
                nv
            }
            Expr::Cond(c, a, b) => {
                let c = self.eval(c, env)?;
                if truthy(&c) {
                    self.eval(a, env)?
                } else {
                    self.eval(b, env)?
                }
            }
            Expr::Member(o, k, opt) => {
                let o = self.eval(o, env)?;
                if *opt && matches!(o, V::Undef | V::Null) {
                    return Ok(V::Undef);
                }
                let k = self.eval(k, env)?;
                self.get_v(&o, &k)?
            }
            Expr::Call(callee, args, opt) => {
                let (this, f) = match &**callee {
                    Expr::Member(o, k, mopt) => {
                        let o = self.eval(o, env)?;
                        if *mopt && matches!(o, V::Undef | V::Null) {
                            return Ok(V::Undef);
                        }
                        let k = self.eval(k, env)?;
                        let f = self.get_v(&o, &k)?;
                        if matches!(f, V::Undef) && !*opt {
                            return type_err(format!("{}.{} is not a function", short(&o), to_str(&k)));
                        }
                        (o, f)
                    }
                    c => (V::Undef, self.eval(c, env)?),
                };
                if *opt && matches!(f, V::Undef | V::Null) {
                    return Ok(V::Undef);
                }
                let args = self.args(args, env)?;
                self.call(&f, this, args)?
            }
            Expr::New(callee, args) => {
                let f = self.eval(callee, env)?;
                let args = self.args(args, env)?;
                match &f {
                    V::Func(fu) => match &**fu {
                        Func::Native(n @ ("Error" | "TypeError")) => {
                            let m = args.first().map(to_str).unwrap_or_default();
                            error_obj(n, &m)
                        }
                        Func::Closure(..) => {
                            let o = obj(IndexMap::new());
                            let r = self.call(&f, o.clone(), args)?;
                            if matches!(r, V::Obj(_) | V::Arr(_)) {
                                r
                            } else {
                                o
                            }
                        }
                        _ => return type_err("This is not a constructor in UMLCraft templates"),
                    },
                    V::Host("Array") => arr(args),
                    V::Host("Object") => obj(IndexMap::new()),
                    _ => return type_err("This is not a constructor in UMLCraft templates"),
                }
            }
            Expr::Spread(_) => return throw("Unexpected spread"),
            Expr::Seq(es) => {
                let mut v = V::Undef;
                for e in es {
                    v = self.eval(e, env)?;
                }
                v
            }
        })
    }

    /// An assignment target with its object and key evaluated once.
    fn place(&mut self, e: &Expr, env: &Env) -> R<Place> {
        match e {
            Expr::Id(n) => Ok(Place::Var(n.clone())),
            Expr::Member(o, k, _) => {
                let o = self.eval(o, env)?;
                let k = self.eval(k, env)?;
                Ok(Place::Prop(o, k))
            }
            _ => Err(Thrown(error_obj("SyntaxError", "Invalid left-hand side in assignment"))),
        }
    }

    fn read_place(&mut self, p: &Place, env: &Env) -> R<V> {
        match p {
            Place::Var(n) => match lookup(env, n) {
                Some(v) => Ok(v),
                None => Err(Thrown(error_obj("ReferenceError", &format!("{n} is not defined")))),
            },
            Place::Prop(o, k) => self.get_v(o, k),
        }
    }

    fn write_place(&mut self, p: &Place, v: V, env: &Env) -> R<()> {
        match p {
            Place::Var(n) => self.bind(&Pat::Id(n.clone()), v, env, false),
            Place::Prop(o, k) => self.put(o, k, v),
        }
    }

    fn args(&mut self, args: &[Expr], env: &Env) -> R<Vec<V>> {
        let mut out = vec![];
        for a in args {
            if let Expr::Spread(x) = a {
                let v = self.eval(x, env)?;
                out.extend(self.iterate(&v)?);
            } else {
                out.push(self.eval(a, env)?);
            }
        }
        Ok(out)
    }

    fn binop(&mut self, op: &str, a: &V, b: &V) -> R<V> {
        Ok(match op {
            "+" => {
                let (pa, pb) = (self.prim(a)?, self.prim(b)?);
                if matches!(pa, V::Str(_)) || matches!(pb, V::Str(_)) {
                    s(&(to_str(&pa) + &to_str(&pb)))
                } else {
                    V::Num(to_num(&pa) + to_num(&pb))
                }
            }
            "-" => V::Num(self.number(a)? - self.number(b)?),
            "*" => V::Num(self.number(a)? * self.number(b)?),
            "/" => V::Num(self.number(a)? / self.number(b)?),
            "%" => V::Num(self.number(a)? % self.number(b)?),
            "**" => V::Num(self.number(a)?.powf(self.number(b)?)),
            "==" => V::Bool(self.loose_eq(a, b)?),
            "!=" => V::Bool(!self.loose_eq(a, b)?),
            "===" => V::Bool(strict_eq(a, b)),
            "!==" => V::Bool(!strict_eq(a, b)),
            "<" | ">" | "<=" | ">=" => {
                let (pa, pb) = (self.prim(a)?, self.prim(b)?);
                if let (V::Str(x), V::Str(y)) = (&pa, &pb) {
                    let o = x.encode_utf16().cmp(y.encode_utf16());
                    V::Bool(match op {
                        "<" => o.is_lt(),
                        ">" => o.is_gt(),
                        "<=" => o.is_le(),
                        _ => o.is_ge(),
                    })
                } else {
                    let (x, y) = (to_num(&pa), to_num(&pb));
                    V::Bool(match op {
                        "<" => x < y,
                        ">" => x > y,
                        "<=" => x <= y,
                        _ => x >= y,
                    })
                }
            }
            "&" => V::Num((to_i32(self.number(a)?) & to_i32(self.number(b)?)) as f64),
            "|" => V::Num((to_i32(self.number(a)?) | to_i32(self.number(b)?)) as f64),
            "^" => V::Num((to_i32(self.number(a)?) ^ to_i32(self.number(b)?)) as f64),
            "<<" => V::Num(to_i32(self.number(a)?).wrapping_shl(to_i32(self.number(b)?) as u32 & 31) as f64),
            ">>" => V::Num(to_i32(self.number(a)?).wrapping_shr(to_i32(self.number(b)?) as u32 & 31) as f64),
            ">>>" => V::Num((to_i32(self.number(a)?) as u32).wrapping_shr(to_i32(self.number(b)?) as u32 & 31) as f64),
            "instanceof" => V::Bool(self.instance_of(a, b)?),
            "in" => {
                let k = to_str(a);
                V::Bool(match b {
                    V::Obj(o) => o.borrow().contains_key(&k),
                    V::Arr(x) => k == "length" || k.parse::<usize>().map(|i| i < x.borrow().len()).unwrap_or(false),
                    V::Elem(_) => !matches!(self.get(b, &k)?, V::Undef),
                    _ => return type_err(format!("Cannot use 'in' operator to search for '{k}' in {}", to_str(b))),
                })
            }
            _ => return throw(format!("Unsupported operator {op}")),
        })
    }

    fn instance_of(&self, a: &V, b: &V) -> R<bool> {
        Ok(match (a, b) {
            (V::Elem(e), V::Type(t)) => self.repo.ty(*e) == &**t || self.repo.kind_of(*e, t),
            (_, V::Type(_)) => false,
            (V::Arr(_), V::Host("Array")) => true,
            (V::Arr(_) | V::Obj(_) | V::Func(_) | V::Elem(_) | V::Regex(_), V::Host("Object")) => true,
            (V::Obj(o), V::Func(f)) if matches!(**f, Func::Native("Error" | "TypeError")) => o.borrow().contains_key("message"),
            (_, V::Func(_) | V::Host(_)) => false,
            _ => return type_err("Right-hand side of 'instanceof' is not callable"),
        })
    }

    fn loose_eq(&mut self, a: &V, b: &V) -> R<bool> {
        Ok(match (a, b) {
            (V::Undef | V::Null, V::Undef | V::Null) => true,
            (V::Undef | V::Null, _) | (_, V::Undef | V::Null) => false,
            (V::Num(_), V::Str(_)) | (V::Str(_), V::Num(_)) | (V::Bool(_), _) | (_, V::Bool(_)) => {
                if matches!((a, b), (V::Bool(x), V::Bool(y)) if x == y) {
                    true
                } else if matches!(a, V::Bool(_)) || matches!(b, V::Bool(_)) || matches!(a, V::Num(_)) || matches!(b, V::Num(_)) {
                    let (pa, pb) = (self.prim(a)?, self.prim(b)?);
                    to_num(&pa) == to_num(&pb)
                } else {
                    strict_eq(a, b)
                }
            }
            (V::Str(_) | V::Num(_), _) | (_, V::Str(_) | V::Num(_)) => {
                let (pa, pb) = (self.prim(a)?, self.prim(b)?);
                match (&pa, &pb) {
                    (V::Str(x), V::Str(y)) => x == y,
                    _ => to_num(&pa) == to_num(&pb),
                }
            }
            _ => strict_eq(a, b),
        })
    }

    /// ToPrimitive: arrays and objects become their strings.
    fn prim(&mut self, v: &V) -> R<V> {
        Ok(match v {
            V::Arr(_) | V::Obj(_) | V::Elem(_) | V::Func(_) | V::Type(_) | V::Host(_) | V::Regex(_) => s(&self.string(v)?),
            _ => v.clone(),
        })
    }

    pub fn number(&mut self, v: &V) -> R<f64> {
        let p = self.prim(v)?;
        Ok(to_num(&p))
    }

    /// String(v), calling a toString method on objects that have one.
    pub fn string(&mut self, v: &V) -> R<String> {
        Ok(match v {
            V::Arr(a) => {
                let items = a.borrow().clone();
                let mut parts = vec![];
                for x in items {
                    parts.push(if matches!(x, V::Undef | V::Null) { String::new() } else { self.string(&x)? });
                }
                parts.join(",")
            }
            V::Obj(o) => {
                let ts = o.borrow().get("toString").cloned();
                match ts {
                    Some(f @ V::Func(_)) => {
                        let r = self.call(&f, v.clone(), vec![])?;
                        to_str(&r)
                    }
                    _ => {
                        let m = o.borrow();
                        match (m.get("name"), m.get("message")) {
                            (Some(n), Some(msg)) if m.len() == 2 => format!("{}: {}", to_str(n), to_str(msg)),
                            _ => "[object Object]".into(),
                        }
                    }
                }
            }
            _ => to_str(v),
        })
    }

    // ---------------------------------------------------------------------------- properties

    fn get_v(&mut self, o: &V, k: &V) -> R<V> {
        match (o, k) {
            (V::Arr(a), V::Num(n)) => Ok(if *n >= 0.0 && n.fract() == 0.0 { a.borrow().get(*n as usize).cloned().unwrap_or(V::Undef) } else { V::Undef }),
            _ => {
                let k = to_str(k);
                self.get(o, &k)
            }
        }
    }

    pub fn get(&mut self, o: &V, k: &str) -> R<V> {
        Ok(match o {
            V::Undef | V::Null => return type_err(format!("Cannot read properties of {} (reading '{k}')", to_str(o))),
            V::Arr(a) => {
                if k == "length" {
                    V::Num(a.borrow().len() as f64)
                } else if let Ok(i) = k.parse::<usize>() {
                    a.borrow().get(i).cloned().unwrap_or(V::Undef)
                } else if ARRAY_METHODS.contains(&k) {
                    method(o, k)
                } else {
                    V::Undef
                }
            }
            V::Str(x) => {
                if k == "length" {
                    V::Num(x.encode_utf16().count() as f64)
                } else if let Ok(i) = k.parse::<usize>() {
                    x.chars().nth(i).map(|c| s(&c.to_string())).unwrap_or(V::Undef)
                } else if STRING_METHODS.contains(&k) {
                    method(o, k)
                } else {
                    V::Undef
                }
            }
            V::Num(_) => {
                if matches!(k, "toFixed" | "toString" | "toPrecision" | "valueOf" | "toLocaleString") {
                    method(o, k)
                } else {
                    V::Undef
                }
            }
            V::Bool(_) => {
                if matches!(k, "toString" | "valueOf") {
                    method(o, k)
                } else {
                    V::Undef
                }
            }
            V::Obj(m) => {
                let v = m.borrow().get(k).cloned();
                match v {
                    Some(v) => v,
                    None if matches!(k, "hasOwnProperty" | "toString") => method(o, k),
                    None => V::Undef,
                }
            }
            V::Func(f) => match k {
                "call" | "apply" | "bind" => method(o, k),
                "name" => s(match &**f {
                    Func::Closure(d, ..) => d.name.as_deref().unwrap_or(""),
                    Func::Native(n) => n,
                    Func::Method(_, n) => n,
                }),
                _ => V::Undef,
            },
            V::Elem(e) => self.elem_get(*e, k),
            V::Type(t) => match k {
                "name" => V::Str(t.clone()),
                _ => V::Undef,
            },
            V::Host(h) => host_get(h, k),
            V::Regex(r) => match k {
                "source" => s(&r.source),
                "flags" => s(&r.flags),
                "global" => V::Bool(r.global),
                "test" | "exec" | "toString" => method(o, k),
                "lastIndex" => V::Num(r.last_index.get() as f64),
                _ => V::Undef,
            },
        })
    }

    fn put(&mut self, o: &V, k: &V, v: V) -> R<()> {
        match o {
            V::Arr(a) => {
                let key = to_str(k);
                if key == "length" {
                    let n = to_num(&v).max(0.0) as usize;
                    a.borrow_mut().resize(n, V::Undef);
                } else if let Ok(i) = key.parse::<usize>() {
                    let mut a = a.borrow_mut();
                    if i >= a.len() {
                        a.resize(i + 1, V::Undef);
                    }
                    a[i] = v;
                }
                Ok(())
            }
            V::Obj(m) => {
                m.borrow_mut().insert(to_str(k), v);
                Ok(())
            }
            V::Elem(_) => type_err("Model elements are read-only in UMLCraft templates"),
            V::Undef | V::Null => type_err(format!("Cannot set properties of {} (setting '{}')", to_str(o), to_str(k))),
            _ => Ok(()),
        }
    }

    // ---------------------------------------------------------------------------- model elements

    /// A JSON value of the file as a template value: references become elements.
    fn json(&self, v: &Json) -> V {
        match v {
            Json::Null => V::Null,
            Json::Bool(b) => V::Bool(*b),
            Json::Number(n) => V::Num(n.as_f64().unwrap_or(0.0)),
            Json::String(x) => s(x),
            Json::Array(a) => arr(a.iter().map(|x| self.json(x)).collect()),
            Json::Object(m) => {
                if m.contains_key("$ref") {
                    return self.repo.by_ref(v).map(V::Elem).unwrap_or(V::Null);
                }
                obj(m.iter().map(|(k, x)| (k.clone(), self.json(x))).collect())
            }
        }
    }

    fn elem_get(&mut self, e: Eid, k: &str) -> V {
        let r = self.repo;
        match k {
            "_id" => return s(&r.id(e)),
            "_parent" => return r.parent(e).map(V::Elem).unwrap_or(V::Null),
            _ => {}
        }
        match r.e(e).fields.get(k) {
            Some(F::One(c)) => return V::Elem(*c),
            Some(F::Many(cs)) => return arr(cs.iter().map(|c| V::Elem(*c)).collect()),
            Some(F::V(v)) => return self.json(v),
            None => {}
        }
        if ELEMENT_METHODS.contains(&k) {
            return method(&V::Elem(e), k);
        }
        // a field the file leaves out has its default (metamodel.json kinds: collections are empty,
        // references null, other fields their default or "")
        if COLLECTION_FIELDS.contains(&k) {
            return arr(vec![]);
        }
        if REFERENCE_FIELDS.contains(&k) {
            return V::Null;
        }
        let ty = r.ty(e);
        if let Some((_, d)) = crate::meta::attrs(ty).into_iter().find(|(n, _)| *n == k) {
            return match d {
                Some(d) => self.json(&d),
                None => s(""),
            };
        }
        if matches!(k, "name" | "documentation") && !crate::meta::known(ty) {
            return s("");
        }
        V::Undef
    }

    fn elem_method(&mut self, e: Eid, name: &str, args: Vec<V>) -> R<V> {
        let r = self.repo;
        let arg = |i: usize| args.get(i).cloned().unwrap_or(V::Undef);
        Ok(match name {
            "getClassName" | "getDisplayClassName" => s(r.ty(e)),
            "getPathname" => {
                // the names from below the project down to this element (core.js:1128-1135)
                let mut parts = vec![];
                let mut cur = Some(e);
                while let Some(c) = cur {
                    if r.parent(c).is_none() {
                        break;
                    }
                    parts.push(to_str(&self.elem_get(c, "name")));
                    cur = r.parent(c);
                }
                if parts.is_empty() {
                    parts.push(to_str(&self.elem_get(e, "name")));
                }
                parts.reverse();
                s(&parts.join("::"))
            }
            "getNodeText" => {
                let n = r.name(e);
                if n.is_empty() {
                    s(&format!("({})", r.ty(e)))
                } else {
                    s(&n)
                }
            }
            "getParentField" => r.field_of(e).map(|f| s(&f)).unwrap_or(V::Null),
            "traverse" | "traverseDepthFirst" => {
                let f = arg(0);
                let mut all = vec![];
                fn walk(r: &Repo, e: Eid, depth_first: bool, out: &mut Vec<Eid>) {
                    if !depth_first {
                        out.push(e);
                    }
                    for (_, c) in r.children(e) {
                        walk(r, c, depth_first, out);
                    }
                    if depth_first {
                        out.push(e);
                    }
                }
                walk(r, e, name == "traverseDepthFirst", &mut all);
                for x in all {
                    self.call(&f, V::Undef, vec![V::Elem(x)])?;
                }
                V::Undef
            }
            "traverseField" | "traverseFieldDepthFirst" => {
                let field = to_str(&arg(0));
                let f = arg(1);
                fn walk(r: &Repo, e: Eid, field: &str, depth_first: bool, out: &mut Vec<Eid>) {
                    if !depth_first {
                        out.push(e);
                    }
                    for c in r.list(e, field) {
                        walk(r, c, field, depth_first, out);
                    }
                    if depth_first {
                        out.push(e);
                    }
                }
                let mut all = vec![];
                walk(r, e, &field, name == "traverseFieldDepthFirst", &mut all);
                for x in all {
                    self.call(&f, V::Undef, vec![V::Elem(x)])?;
                }
                V::Undef
            }
            "traverseUp" => {
                let f = arg(0);
                let mut cur = Some(e);
                while let Some(c) = cur {
                    self.call(&f, V::Undef, vec![V::Elem(c)])?;
                    cur = r.parent(c);
                }
                V::Undef
            }
            "getTag" | "getTagValue" => {
                let n = to_str(&arg(0));
                match r.list(e, "tags").into_iter().find(|t| r.name(*t) == n) {
                    Some(t) if name == "getTag" => V::Elem(t),
                    Some(t) => self.elem_get(t, "value"),
                    None => V::Undef,
                }
            }
            "getGeneralElements" | "getSpecialElements" => {
                // the other ends of generalizations (and with true, interface realizations)
                // (uml/elements.js:1215-1249)
                let up = name == "getGeneralElements";
                let with_ifaces = truthy(&arg(0));
                let mut out = vec![];
                for x in r.all() {
                    let ty = r.ty(x);
                    if ty == "UMLGeneralization" || (with_ifaces && ty == "UMLInterfaceRealization") {
                        let (from, to) = if up { ("source", "target") } else { ("target", "source") };
                        if r.refd(x, from) == Some(e) {
                            if let Some(o) = r.refd(x, to) {
                                out.push(V::Elem(o));
                            }
                        }
                    }
                }
                arr(out)
            }
            "getReturnParameter" => r.list(e, "parameters").into_iter().find(|p| r.str(*p, "direction") == "return").map(V::Elem).unwrap_or(V::Null),
            "getNonReturnParameters" => arr(r.list(e, "parameters").into_iter().filter(|p| r.str(*p, "direction") != "return").map(V::Elem).collect()),
            "getVisibilityString" => match r.str(e, "visibility").as_str() {
                "public" => s("+"),
                "protected" => s("#"),
                "private" => s("-"),
                "package" => s("~"),
                _ => V::Undef,
            },
            "getStereotypeString" => match self.elem_get(e, "stereotype") {
                V::Str(x) if !x.is_empty() => s(&format!("«{x}»")),
                V::Elem(st) => s(&format!("«{}»", r.name(st))),
                _ => s(""),
            },
            _ => V::Undef,
        })
    }

    // ---------------------------------------------------------------------------- calls

    pub fn call(&mut self, f: &V, this: V, args: Vec<V>) -> R<V> {
        self.tick()?;
        let V::Func(fu) = f else {
            if let V::Host(h @ ("String" | "Number" | "Boolean" | "Array" | "Object")) = f {
                return self.native(h, args);
            }
            return type_err(format!("{} is not a function", short(f)));
        };
        match &**fu {
            Func::Closure(def, env, lex_this) => {
                let e = scope(Some(env.clone()), true);
                declare(&e, "this", lex_this.clone().unwrap_or(this));
                for (i, p) in def.params.iter().enumerate() {
                    self.bind(p, args.get(i).cloned().unwrap_or(V::Undef), &e, true)?;
                }
                if let Some(rp) = &def.rest {
                    let rest = args.get(def.params.len()..).map(|x| x.to_vec()).unwrap_or_default();
                    self.bind(rp, arr(rest), &e, true)?;
                }
                if !def.arrow {
                    declare(&e, "arguments", arr(args.clone()));
                }
                match &def.body {
                    Body::Expr(x) => self.eval(x, &e),
                    Body::Block(b) => {
                        self.hoist(b, &e);
                        match self.block(b, &e)? {
                            Ctl::Ret(v) => Ok(v),
                            _ => Ok(V::Undef),
                        }
                    }
                }
            }
            Func::Method(this, name) => self.call_method(this, name, args),
            Func::Native(n) => self.native(n, args),
        }
    }

    fn call_method(&mut self, this: &V, name: &str, args: Vec<V>) -> R<V> {
        if let ("call" | "apply" | "bind", V::Func(_)) = (name, this) {
            let t = args.first().cloned().unwrap_or(V::Undef);
            return match name {
                "call" => self.call(this, t, args.get(1..).map(|x| x.to_vec()).unwrap_or_default()),
                "apply" => {
                    let a = match args.get(1) {
                        Some(x @ V::Arr(_)) => self.iterate(x)?,
                        _ => vec![],
                    };
                    self.call(this, t, a)
                }
                _ => Ok(this.clone()),
            };
        }
        match this {
            V::Arr(a) => self.array_method(a, name, args),
            V::Str(x) => self.string_method(x, name, args),
            V::Num(n) => {
                let a0 = args.first().cloned().unwrap_or(V::Undef);
                Ok(match name {
                    "toFixed" => {
                        let d = if matches!(a0, V::Undef) { 0 } else { to_num(&a0).clamp(0.0, 100.0) as usize };
                        s(&format!("{:.*}", d, n))
                    }
                    "toPrecision" => {
                        let p = to_num(&a0).clamp(1.0, 100.0) as usize;
                        let digits = if *n == 0.0 { 0 } else { n.abs().log10().floor() as i64 };
                        let dec = (p as i64 - 1 - digits).max(0) as usize;
                        s(&format!("{:.*}", dec, n))
                    }
                    "toString" => {
                        let radix = if matches!(a0, V::Undef) { 10 } else { to_num(&a0) as u32 };
                        if radix == 10 || n.fract() != 0.0 {
                            s(&num_str(*n))
                        } else {
                            s(&radix_str(*n as i64, radix))
                        }
                    }
                    _ => V::Num(*n),
                })
            }
            V::Bool(b) => Ok(if name == "toString" { s(if *b { "true" } else { "false" }) } else { V::Bool(*b) }),
            V::Obj(o) => match name {
                "hasOwnProperty" => {
                    let k = to_str(&args.first().cloned().unwrap_or(V::Undef));
                    Ok(V::Bool(o.borrow().contains_key(&k)))
                }
                _ => Ok(s(&self.string(this)?)),
            },
            V::Elem(e) => self.elem_method(*e, name, args),
            V::Regex(r) => {
                let x = args.first().map(to_str).unwrap_or_else(|| "undefined".into());
                Ok(match name {
                    "test" => V::Bool(self.regex_exec(r, &x).is_some()),
                    "exec" => match self.regex_exec(r, &x) {
                        Some(m) => self.match_array(&m, &x),
                        None => V::Null,
                    },
                    _ => s(&format!("/{}/{}", r.source, r.flags)),
                })
            }
            V::Host(h) => self.native_host(h, name, args),
            _ => type_err(format!("{name} is not a function")),
        }
    }

    // ---------------------------------------------------------------------------- regular expressions

    /// A match from lastIndex for a global regex (moving it), else from the start.
    fn regex_exec(&self, r: &crate::jsregex::Regex, x: &str) -> Option<crate::jsregex::Match> {
        let chars: Vec<char> = x.chars().collect();
        let start = if r.global || r.sticky { r.last_index.get() } else { 0 };
        if start > chars.len() {
            r.last_index.set(0);
            return None;
        }
        let m = r.find_at(&chars, start);
        if r.global || r.sticky {
            match &m {
                Some(m) => r.last_index.set(if m.end == m.start { m.end + 1 } else { m.end }),
                None => r.last_index.set(0),
            }
        }
        m
    }

    fn match_array(&self, m: &crate::jsregex::Match, x: &str) -> V {
        let chars: Vec<char> = x.chars().collect();
        let mut items = vec![];
        for g in &m.groups {
            items.push(match g {
                Some((a, b)) => s(&chars[*a..*b].iter().collect::<String>()),
                None => V::Undef,
            });
        }
        arr(items)
    }

    /// The replacement text for one match: `$&`, `$1`… or a function's result.
    fn replacement(&mut self, rep: &V, m: &crate::jsregex::Match, chars: &[char]) -> R<String> {
        let group = |i: usize| m.groups.get(i).and_then(|g| g.map(|(a, b)| chars[a..b].iter().collect::<String>()));
        if let V::Func(_) = rep {
            let mut args: Vec<V> = (0..m.groups.len()).map(|i| group(i).map(|g| s(&g)).unwrap_or(V::Undef)).collect();
            args.push(V::Num(m.start as f64));
            args.push(s(&chars.iter().collect::<String>()));
            let r = self.call(rep, V::Undef, args)?;
            return self.string(&r);
        }
        let t: Vec<char> = to_str(rep).chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < t.len() {
            if t[i] == '$' && i + 1 < t.len() {
                match t[i + 1] {
                    '$' => {
                        out.push('$');
                        i += 2;
                        continue;
                    }
                    '&' => {
                        out.push_str(&group(0).unwrap_or_default());
                        i += 2;
                        continue;
                    }
                    '`' => {
                        out.extend(&chars[..m.start]);
                        i += 2;
                        continue;
                    }
                    '\'' => {
                        out.extend(&chars[m.end..]);
                        i += 2;
                        continue;
                    }
                    d if d.is_ascii_digit() => {
                        let mut n = d.to_digit(10).unwrap() as usize;
                        let mut len = 2;
                        if let Some(d2) = t.get(i + 2).and_then(|c| c.to_digit(10)) {
                            if n * 10 + (d2 as usize) < m.groups.len() {
                                n = n * 10 + d2 as usize;
                                len = 3;
                            }
                        }
                        if n > 0 && n < m.groups.len() {
                            out.push_str(&group(n).unwrap_or_default());
                            i += len;
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            out.push(t[i]);
            i += 1;
        }
        Ok(out)
    }

    // ---------------------------------------------------------------------------- strings

    fn string_method(&mut self, x: &Rc<str>, name: &str, args: Vec<V>) -> R<V> {
        let a = |i: usize| args.get(i).cloned().unwrap_or(V::Undef);
        let st = |i: usize| to_str(&args.get(i).cloned().unwrap_or(V::Undef));
        let chars: Vec<char> = x.chars().collect();
        let len = chars.len() as f64;
        let idx = |v: &V, def: f64| -> usize {
            let n = if matches!(v, V::Undef) { def } else { to_num(v) };
            let n = if n.is_nan() { 0.0 } else { n };
            let n = if n < 0.0 { (len + n).max(0.0) } else { n.min(len) };
            n as usize
        };
        let sub = |a: usize, b: usize| -> String { if a < b { chars[a..b].iter().collect() } else { String::new() } };
        Ok(match name {
            "toUpperCase" | "toLocaleUpperCase" => s(&x.to_uppercase()),
            "toLowerCase" | "toLocaleLowerCase" => s(&x.to_lowercase()),
            "trim" => s(x.trim()),
            "trimStart" | "trimLeft" => s(x.trim_start()),
            "trimEnd" | "trimRight" => s(x.trim_end()),
            "toString" | "valueOf" => V::Str(x.clone()),
            "charAt" => {
                let i = to_num(&a(0));
                let i = if i.is_nan() { 0.0 } else { i };
                s(&chars.get(i as usize).filter(|_| i >= 0.0).map(|c| c.to_string()).unwrap_or_default())
            }
            "charCodeAt" | "codePointAt" => {
                let i = to_num(&a(0));
                let i = if i.is_nan() { 0.0 } else { i };
                chars.get(i as usize).filter(|_| i >= 0.0).map(|c| V::Num(*c as u32 as f64)).unwrap_or(V::Num(f64::NAN))
            }
            "at" => {
                let i = to_num(&a(0));
                let i = if i < 0.0 { len + i } else { i };
                chars.get(i as usize).filter(|_| i >= 0.0).map(|c| s(&c.to_string())).unwrap_or(V::Undef)
            }
            "indexOf" | "lastIndexOf" | "includes" | "startsWith" | "endsWith" => {
                let needle: Vec<char> = st(0).chars().collect();
                let found = |from: usize| -> Option<usize> { (from..=chars.len().saturating_sub(needle.len())).find(|i| chars[*i..].starts_with(&needle)).filter(|_| needle.len() <= chars.len()) };
                match name {
                    "indexOf" => V::Num(found(idx(&a(1), 0.0)).map(|i| i as f64).unwrap_or(-1.0)),
                    "lastIndexOf" => {
                        let r = if needle.len() > chars.len() { None } else { (0..=chars.len() - needle.len()).rev().find(|i| chars[*i..].starts_with(&needle)) };
                        V::Num(r.map(|i| i as f64).unwrap_or(-1.0))
                    }
                    "includes" => V::Bool(found(idx(&a(1), 0.0)).is_some()),
                    "startsWith" => V::Bool(chars[idx(&a(1), 0.0)..].starts_with(&needle)),
                    _ => V::Bool(chars[..idx(&a(1), len)].ends_with(&needle)),
                }
            }
            "slice" => s(&sub(idx(&a(0), 0.0), idx(&a(1), len))),
            "substring" => {
                let f = |v: &V, d: f64| -> usize {
                    let n = if matches!(v, V::Undef) { d } else { to_num(v) };
                    if n.is_nan() { 0 } else { n.clamp(0.0, len) as usize }
                };
                let (p, q) = (f(&a(0), 0.0), f(&a(1), len));
                s(&sub(p.min(q), p.max(q)))
            }
            "substr" => {
                let st0 = idx(&a(0), 0.0);
                let n = if matches!(a(1), V::Undef) { len } else { to_num(&a(1)).max(0.0) };
                s(&sub(st0, (st0 + n as usize).min(chars.len())))
            }
            "split" => {
                let lim = if matches!(a(1), V::Undef) { usize::MAX } else { to_num(&a(1)) as usize };
                let parts: Vec<V> = match a(0) {
                    V::Undef => vec![V::Str(x.clone())],
                    V::Regex(r) => {
                        // the SplitMatcher loop of the standard: an empty match only splits between
                        // characters
                        let mut out = vec![];
                        if chars.is_empty() {
                            if r.match_at(&chars, 0).is_none() {
                                out.push(s(""));
                            }
                        } else {
                            let (mut p, mut q) = (0, 0);
                            while q < chars.len() {
                                match r.match_at(&chars, q) {
                                    Some(m) if m.end != p && !(m.end == m.start && m.end >= chars.len()) => {
                                        out.push(s(&sub(p, q)));
                                        for g in m.groups.iter().skip(1) {
                                            out.push(g.map(|(x, y)| s(&sub(x, y))).unwrap_or(V::Undef));
                                        }
                                        p = m.end;
                                        q = if m.end > q { m.end } else { q + 1 };
                                    }
                                    _ => q += 1,
                                }
                            }
                            out.push(s(&sub(p, chars.len())));
                        }
                        out
                    }
                    sep => {
                        let sep = to_str(&sep);
                        if sep.is_empty() {
                            chars.iter().map(|c| s(&c.to_string())).collect()
                        } else {
                            x.split(sep.as_str()).map(s).collect()
                        }
                    }
                };
                arr(parts.into_iter().take(lim).collect())
            }
            "replace" | "replaceAll" => {
                let rep = a(1);
                match a(0) {
                    V::Regex(r) => {
                        let all = r.global || name == "replaceAll";
                        let mut out = String::new();
                        let mut last = 0;
                        let mut at = 0;
                        while at <= chars.len() {
                            let Some(m) = r.find_at(&chars, at) else { break };
                            out.push_str(&sub(last, m.start));
                            out.push_str(&self.replacement(&rep, &m, &chars)?);
                            last = m.end;
                            if !all {
                                break;
                            }
                            at = if m.end > m.start { m.end } else { m.end + 1 };
                        }
                        out.push_str(&sub(last, chars.len()));
                        s(&out)
                    }
                    pat => {
                        let pat: Vec<char> = to_str(&pat).chars().collect();
                        let mut out = String::new();
                        let mut last = 0;
                        let mut i = 0;
                        while i <= chars.len() {
                            if chars[i..].starts_with(&pat) {
                                let m = crate::jsregex::Match { start: i, end: i + pat.len(), groups: vec![Some((i, i + pat.len()))] };
                                out.push_str(&sub(last, i));
                                out.push_str(&self.replacement(&rep, &m, &chars)?);
                                last = i + pat.len();
                                if name == "replace" {
                                    break;
                                }
                                i += pat.len().max(1);
                            } else {
                                i += 1;
                            }
                        }
                        out.push_str(&sub(last, chars.len()));
                        s(&out)
                    }
                }
            }
            "match" | "matchAll" | "search" => {
                let r = match a(0) {
                    V::Regex(r) => r,
                    v => Rc::new(crate::jsregex::Regex::new(&to_str(&v), "").map_err(|m| Thrown(error_obj("SyntaxError", &m)))?),
                };
                if name == "search" {
                    return Ok(V::Num(r.find_at(&chars, 0).map(|m| m.start as f64).unwrap_or(-1.0)));
                }
                if r.global || name == "matchAll" {
                    let mut out = vec![];
                    let mut at = 0;
                    while at <= chars.len() {
                        let Some(m) = r.find_at(&chars, at) else { break };
                        at = if m.end > m.start { m.end } else { m.end + 1 };
                        out.push(if name == "matchAll" { self.match_array(&m, x) } else { s(&sub(m.start, m.end)) });
                    }
                    if out.is_empty() && name == "match" {
                        V::Null
                    } else {
                        arr(out)
                    }
                } else {
                    match r.find_at(&chars, 0) {
                        Some(m) => self.match_array(&m, x),
                        None => V::Null,
                    }
                }
            }
            "padStart" | "padEnd" => {
                let n = to_num(&a(0)) as usize;
                let fill = if matches!(a(1), V::Undef) { " ".to_string() } else { st(1) };
                if n <= chars.len() || fill.is_empty() {
                    V::Str(x.clone())
                } else {
                    let pad: String = fill.chars().cycle().take(n - chars.len()).collect();
                    s(&if name == "padStart" { pad + x } else { x.to_string() + &pad })
                }
            }
            "repeat" => s(&x.repeat(to_num(&a(0)).max(0.0) as usize)),
            "concat" => {
                let mut out = x.to_string();
                for v in &args {
                    out.push_str(&self.string(v)?);
                }
                s(&out)
            }
            "localeCompare" => {
                let o = x.to_lowercase().cmp(&st(0).to_lowercase()).then(x.as_ref().cmp(st(0).as_str()));
                V::Num(o as i32 as f64)
            }
            "normalize" => V::Str(x.clone()),
            _ => return type_err(format!("{name} is not a function")),
        })
    }

    // ---------------------------------------------------------------------------- arrays

    fn array_method(&mut self, a: &Arr, name: &str, args: Vec<V>) -> R<V> {
        let arg = |i: usize| args.get(i).cloned().unwrap_or(V::Undef);
        let this = V::Arr(a.clone());
        let items = a.borrow().clone();
        let len = items.len() as f64;
        let idx = |v: &V, def: f64| -> usize {
            let n = if matches!(v, V::Undef) { def } else { to_num(v) };
            let n = if n.is_nan() { 0.0 } else { n };
            (if n < 0.0 { (len + n).max(0.0) } else { n.min(len) }) as usize
        };
        Ok(match name {
            "push" => {
                a.borrow_mut().extend(args);
                V::Num(a.borrow().len() as f64)
            }
            "pop" => a.borrow_mut().pop().unwrap_or(V::Undef),
            "shift" => {
                if items.is_empty() {
                    V::Undef
                } else {
                    a.borrow_mut().remove(0)
                }
            }
            "unshift" => {
                let mut v = args;
                v.extend(items);
                *a.borrow_mut() = v;
                V::Num(a.borrow().len() as f64)
            }
            "forEach" | "map" | "filter" | "find" | "findIndex" | "findLast" | "findLastIndex" | "some" | "every" | "flatMap" => {
                let f = arg(0);
                let mut out = vec![];
                let order: Vec<usize> = if name.starts_with("findLast") { (0..items.len()).rev().collect() } else { (0..items.len()).collect() };
                for i in order {
                    let x = items[i].clone();
                    let r = self.call(&f, arg(1), vec![x.clone(), V::Num(i as f64), this.clone()])?;
                    match name {
                        "map" => out.push(r),
                        "flatMap" => match r {
                            V::Arr(inner) => out.extend(inner.borrow().iter().cloned()),
                            r => out.push(r),
                        },
                        "filter" if truthy(&r) => out.push(x),
                        "find" | "findLast" if truthy(&r) => return Ok(x),
                        "findIndex" | "findLastIndex" if truthy(&r) => return Ok(V::Num(i as f64)),
                        "some" if truthy(&r) => return Ok(V::Bool(true)),
                        "every" if !truthy(&r) => return Ok(V::Bool(false)),
                        _ => {}
                    }
                }
                match name {
                    "map" | "filter" | "flatMap" => arr(out),
                    "find" | "findLast" | "forEach" => V::Undef,
                    "findIndex" | "findLastIndex" => V::Num(-1.0),
                    "some" => V::Bool(false),
                    _ => V::Bool(true),
                }
            }
            "reduce" | "reduceRight" => {
                let f = arg(0);
                let order: Vec<usize> = if name == "reduceRight" { (0..items.len()).rev().collect() } else { (0..items.len()).collect() };
                let mut it = order.into_iter();
                let mut acc = if args.len() > 1 {
                    arg(1)
                } else {
                    match it.next() {
                        Some(i) => items[i].clone(),
                        None => return type_err("Reduce of empty array with no initial value"),
                    }
                };
                for i in it {
                    acc = self.call(&f, V::Undef, vec![acc, items[i].clone(), V::Num(i as f64), this.clone()])?;
                }
                acc
            }
            "join" => {
                let sep = if matches!(arg(0), V::Undef) { ",".to_string() } else { to_str(&arg(0)) };
                let mut parts = vec![];
                for x in &items {
                    parts.push(if matches!(x, V::Undef | V::Null) { String::new() } else { self.string(x)? });
                }
                s(&parts.join(&sep))
            }
            "toString" => s(&self.string(&this)?),
            "indexOf" | "lastIndexOf" | "includes" => {
                let x = arg(0);
                let pos = if name == "lastIndexOf" { items.iter().rposition(|y| strict_eq(y, &x)) } else { items.iter().enumerate().skip(idx(&arg(1), 0.0)).find(|(_, y)| if name == "includes" { same_value_zero(y, &x) } else { strict_eq(y, &x) }).map(|(i, _)| i) };
                if name == "includes" {
                    V::Bool(pos.is_some())
                } else {
                    V::Num(pos.map(|i| i as f64).unwrap_or(-1.0))
                }
            }
            "slice" => {
                let (p, q) = (idx(&arg(0), 0.0), idx(&arg(1), len));
                arr(if p < q { items[p..q].to_vec() } else { vec![] })
            }
            "splice" => {
                let st = idx(&arg(0), 0.0);
                let n = if args.len() < 2 { items.len() - st } else { (to_num(&arg(1)).max(0.0) as usize).min(items.len() - st) };
                let mut v = items;
                let removed: Vec<V> = v.splice(st..st + n, args.into_iter().skip(2)).collect();
                *a.borrow_mut() = v;
                arr(removed)
            }
            "concat" => {
                let mut out = items;
                for x in args {
                    match x {
                        V::Arr(o) => out.extend(o.borrow().iter().cloned()),
                        x => out.push(x),
                    }
                }
                arr(out)
            }
            "reverse" => {
                a.borrow_mut().reverse();
                this
            }
            "sort" => {
                let f = arg(0);
                let sorted = self.sort(items, &f)?;
                *a.borrow_mut() = sorted;
                this
            }
            "flat" => {
                let depth = if matches!(arg(0), V::Undef) { 1.0 } else { to_num(&arg(0)) };
                fn flat(v: &[V], d: f64, out: &mut Vec<V>) {
                    for x in v {
                        match x {
                            V::Arr(i) if d >= 1.0 => flat(&i.borrow(), d - 1.0, out),
                            x => out.push(x.clone()),
                        }
                    }
                }
                let mut out = vec![];
                flat(&items, depth, &mut out);
                arr(out)
            }
            "at" => {
                let i = to_num(&arg(0));
                let i = if i < 0.0 { len + i } else { i };
                items.get(i as usize).filter(|_| i >= 0.0).cloned().unwrap_or(V::Undef)
            }
            "fill" => {
                let x = arg(0);
                let (p, q) = (idx(&arg(1), 0.0), idx(&arg(2), len));
                let mut v = items;
                for slot in v.iter_mut().take(q).skip(p) {
                    *slot = x.clone();
                }
                *a.borrow_mut() = v;
                this
            }
            "keys" => arr((0..items.len()).map(|i| V::Num(i as f64)).collect()),
            "entries" => arr(items.into_iter().enumerate().map(|(i, x)| arr(vec![V::Num(i as f64), x])).collect()),
            "values" => arr(items),
            _ => return type_err(format!("{name} is not a function")),
        })
    }

    /// A stable sort with a comparator function, or by strings without one.
    fn sort(&mut self, items: Vec<V>, f: &V) -> R<Vec<V>> {
        // merge sort, so that the comparator may fail
        if items.len() <= 1 {
            return Ok(items);
        }
        let mid = items.len() / 2;
        let right = items[mid..].to_vec();
        let mut left = items;
        left.truncate(mid);
        let l = self.sort(left, f)?;
        let r = self.sort(right, f)?;
        let mut out = Vec::with_capacity(l.len() + r.len());
        let (mut i, mut j) = (0, 0);
        while i < l.len() && j < r.len() {
            let le = self.compare(&l[i], &r[j], f)? <= 0.0;
            if le {
                out.push(l[i].clone());
                i += 1;
            } else {
                out.push(r[j].clone());
                j += 1;
            }
        }
        out.extend_from_slice(&l[i..]);
        out.extend_from_slice(&r[j..]);
        Ok(out)
    }

    fn compare(&mut self, a: &V, b: &V, f: &V) -> R<f64> {
        // undefined sorts last
        match (a, b) {
            (V::Undef, V::Undef) => return Ok(0.0),
            (V::Undef, _) => return Ok(1.0),
            (_, V::Undef) => return Ok(-1.0),
            _ => {}
        }
        if let V::Undef = f {
            let (x, y) = (self.string(a)?, self.string(b)?);
            return Ok(x.encode_utf16().cmp(y.encode_utf16()) as i32 as f64);
        }
        let r = self.call(f, V::Undef, vec![a.clone(), b.clone()])?;
        let n = self.number(&r)?;
        Ok(if n.is_nan() { 0.0 } else { n })
    }

    // ---------------------------------------------------------------------------- built-ins

    fn native(&mut self, n: &str, args: Vec<V>) -> R<V> {
        let a = |i: usize| args.get(i).cloned().unwrap_or(V::Undef);
        Ok(match n {
            "String" => {
                if args.is_empty() {
                    s("")
                } else {
                    s(&self.string(&a(0))?)
                }
            }
            "Number" => V::Num(if args.is_empty() { 0.0 } else { self.number(&a(0))? }),
            "Boolean" => V::Bool(truthy(&a(0))),
            "Array" => arr(args),
            "Object" => match a(0) {
                v @ (V::Obj(_) | V::Arr(_)) => v,
                _ => obj(IndexMap::new()),
            },
            "parseInt" => {
                let t = self.string(&a(0))?;
                let t = t.trim();
                let radix = if matches!(a(1), V::Undef) { 0 } else { to_num(&a(1)) as u32 };
                let (neg, t) = match t.strip_prefix('-') {
                    Some(r) => (true, r),
                    None => (false, t.strip_prefix('+').unwrap_or(t)),
                };
                let (radix, t) = if (radix == 0 || radix == 16) && (t.starts_with("0x") || t.starts_with("0X")) { (16, &t[2..]) } else { (if radix == 0 { 10 } else { radix }, t) };
                let digits: String = t.chars().take_while(|c| c.to_digit(radix.clamp(2, 36)).is_some()).collect();
                if digits.is_empty() {
                    V::Num(f64::NAN)
                } else {
                    let v = digits.chars().fold(0f64, |acc, c| acc * radix as f64 + c.to_digit(radix).unwrap() as f64);
                    V::Num(if neg { -v } else { v })
                }
            }
            "parseFloat" => {
                let t = self.string(&a(0))?;
                let t = t.trim_start();
                let mut end = 0;
                let b: Vec<char> = t.chars().collect();
                let mut seen_dot = false;
                let mut seen_e = false;
                while end < b.len() {
                    let c = b[end];
                    if c.is_ascii_digit() || ((c == '-' || c == '+') && (end == 0 || matches!(b[end - 1], 'e' | 'E'))) {
                        end += 1;
                    } else if c == '.' && !seen_dot && !seen_e {
                        seen_dot = true;
                        end += 1;
                    } else if (c == 'e' || c == 'E') && !seen_e && end > 0 {
                        seen_e = true;
                        end += 1;
                    } else {
                        break;
                    }
                }
                let mut txt: String = b[..end].iter().collect();
                while !txt.is_empty() && txt.parse::<f64>().is_err() {
                    txt.pop();
                }
                if t.starts_with("Infinity") {
                    V::Num(f64::INFINITY)
                } else {
                    V::Num(txt.parse().unwrap_or(f64::NAN))
                }
            }
            "isNaN" => V::Bool(self.number(&a(0))?.is_nan()),
            "isFinite" => V::Bool(self.number(&a(0))?.is_finite()),
            "filenamify" => s(&crate::export::filenamify(&self.string(&a(0))?)),
            "Error" | "TypeError" => {
                let m = if matches!(a(0), V::Undef) { String::new() } else { to_str(&a(0)) };
                error_obj(n, &m)
            }
            "encodeURIComponent" => {
                let t = self.string(&a(0))?;
                let mut out = String::new();
                for b in t.bytes() {
                    if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
                        out.push(b as char);
                    } else {
                        out.push_str(&format!("%{b:02X}"));
                    }
                }
                s(&out)
            }
            "decodeURIComponent" => {
                let t = self.string(&a(0))?;
                let b = t.as_bytes();
                let mut out = vec![];
                let mut i = 0;
                while i < b.len() {
                    if b[i] == b'%' && i + 2 < b.len() {
                        if let Some(x) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                            out.push(x);
                            i += 3;
                            continue;
                        }
                    }
                    out.push(b[i]);
                    i += 1;
                }
                s(&String::from_utf8_lossy(&out))
            }
            "__append" => {
                let v = a(0);
                if !matches!(v, V::Undef | V::Null) {
                    let t = self.string(&v)?;
                    if let Some(o) = self.out.last_mut() {
                        o.push_str(&t);
                    }
                }
                V::Undef
            }
            "__escape" => {
                let v = a(0);
                if matches!(v, V::Undef | V::Null) {
                    s("")
                } else {
                    s(&escape_xml(&self.string(&v)?))
                }
            }
            "include" => {
                let Some(hook) = self.include else { return throw("include is not available here") };
                let p = self.string(&a(0))?;
                hook(self, &p, a(1))?
            }
            _ => {
                if let Some((h, m)) = n.rsplit_once('.') {
                    let h: &'static str = HOSTS.iter().find(|x| **x == h).copied().unwrap_or("");
                    return self.native_host(h, m, args);
                }
                return type_err(format!("{n} is not a function"));
            }
        })
    }

    fn native_host(&mut self, h: &str, m: &str, args: Vec<V>) -> R<V> {
        let a = |i: usize| args.get(i).cloned().unwrap_or(V::Undef);
        let repo = self.repo;
        Ok(match (h, m) {
            ("Math", _) => {
                let mut nums = vec![];
                for x in &args {
                    nums.push(self.number(x)?);
                }
                let x = nums.first().copied().unwrap_or(f64::NAN);
                let y = nums.get(1).copied().unwrap_or(f64::NAN);
                V::Num(match m {
                    "min" => nums.iter().copied().fold(f64::INFINITY, |p, q| if p.is_nan() || q.is_nan() { f64::NAN } else { p.min(q) }),
                    "max" => nums.iter().copied().fold(f64::NEG_INFINITY, |p, q| if p.is_nan() || q.is_nan() { f64::NAN } else { p.max(q) }),
                    "floor" => x.floor(),
                    "ceil" => x.ceil(),
                    "round" => (x + 0.5).floor(),
                    "trunc" => x.trunc(),
                    "abs" => x.abs(),
                    "sign" => {
                        if x == 0.0 || x.is_nan() {
                            x
                        } else {
                            x.signum()
                        }
                    }
                    "pow" => x.powf(y),
                    "sqrt" => x.sqrt(),
                    "cbrt" => x.cbrt(),
                    "exp" => x.exp(),
                    "log" => x.ln(),
                    "log10" => x.log10(),
                    "log2" => x.log2(),
                    "sin" => x.sin(),
                    "cos" => x.cos(),
                    "tan" => x.tan(),
                    "atan" => x.atan(),
                    "atan2" => x.atan2(y),
                    "hypot" => nums.iter().map(|v| v * v).sum::<f64>().sqrt(),
                    "random" => {
                        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
                        (t % 1_000_000) as f64 / 1_000_000.0
                    }
                    _ => return type_err(format!("Math.{m} is not a function")),
                })
            }
            ("JSON", "stringify") => {
                let indent = match a(2) {
                    V::Num(n) => " ".repeat(n.clamp(0.0, 10.0) as usize),
                    V::Str(x) => x.chars().take(10).collect(),
                    _ => String::new(),
                };
                match self.to_json(&a(0), 0)? {
                    Some(j) => {
                        if indent.is_empty() {
                            s(&serde_json::to_string(&j).unwrap_or_default())
                        } else {
                            let mut buf = vec![];
                            let fmt = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                            let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
                            serde::Serialize::serialize(&j, &mut ser).ok();
                            s(&String::from_utf8_lossy(&buf))
                        }
                    }
                    None => V::Undef,
                }
            }
            ("JSON", "parse") => {
                let t = self.string(&a(0))?;
                match serde_json::from_str::<Json>(&t) {
                    Ok(j) => self.json_plain(&j),
                    Err(e) => return Err(Thrown(error_obj("SyntaxError", &e.to_string()))),
                }
            }
            ("Object", "keys") => arr(self.keys(&a(0)).into_iter().map(|k| s(&k)).collect()),
            ("Object", "values" | "entries") => {
                let o = a(0);
                let mut out = vec![];
                for k in self.keys(&o) {
                    let v = self.get(&o, &k)?;
                    out.push(if m == "values" { v } else { arr(vec![s(&k), v]) });
                }
                arr(out)
            }
            ("Object", "assign") => {
                let target = a(0);
                for src in args.iter().skip(1) {
                    for k in self.keys(src) {
                        let v = self.get(src, &k)?;
                        self.put(&target, &s(&k), v)?;
                    }
                }
                target
            }
            ("Object", "fromEntries") => {
                let mut out = IndexMap::new();
                for e in self.iterate(&a(0))? {
                    let k = self.get(&e, "0")?;
                    let v = self.get(&e, "1")?;
                    out.insert(to_str(&k), v);
                }
                obj(out)
            }
            ("Array", "isArray") => V::Bool(matches!(a(0), V::Arr(_))),
            ("Array", "from") => {
                let items = match a(0) {
                    V::Obj(o) => {
                        let n = o.borrow().get("length").map(to_num).unwrap_or(0.0) as usize;
                        vec![V::Undef; n]
                    }
                    v => self.iterate(&v)?,
                };
                if let f @ V::Func(_) = a(1) {
                    let mut out = vec![];
                    for (i, x) in items.into_iter().enumerate() {
                        out.push(self.call(&f, V::Undef, vec![x, V::Num(i as f64)])?);
                    }
                    arr(out)
                } else {
                    arr(items)
                }
            }
            ("Array", "of") => arr(args),
            ("Number", "isInteger") => V::Bool(matches!(a(0), V::Num(n) if n.is_finite() && n.fract() == 0.0)),
            ("Number", "isFinite") => V::Bool(matches!(a(0), V::Num(n) if n.is_finite())),
            ("Number", "isNaN") => V::Bool(matches!(a(0), V::Num(n) if n.is_nan())),
            ("Number", "parseInt") => self.native("parseInt", args)?,
            ("Number", "parseFloat") => self.native("parseFloat", args)?,
            ("String", "fromCharCode") => s(&args.iter().filter_map(|x| char::from_u32(to_num(x) as u32)).collect::<String>()),
            ("console", _) => {
                let mut parts = vec![];
                for x in &args {
                    parts.push(self.string(x)?);
                }
                eprintln!("{}", parts.join(" "));
                V::Undef
            }
            ("app.repository", "select") => {
                let sel = self.string(&a(0))?;
                match crate::selector::select(repo, &sel) {
                    Ok(v) => arr(v.into_iter().map(V::Elem).collect()),
                    Err(e) => return throw(e),
                }
            }
            ("app.repository", "get") => {
                let id = self.string(&a(0))?;
                repo.by_id.get(&id).copied().filter(|e| repo.alive(*e)).map(V::Elem).unwrap_or(V::Undef)
            }
            ("app.repository", "getInstancesOf") => {
                let names: Vec<String> = match a(0) {
                    V::Arr(x) => x.borrow().iter().map(to_str).collect(),
                    v => vec![to_str(&v)],
                };
                arr(repo.all().filter(|e| names.iter().any(|n| repo.ty(*e) == n || repo.kind_of(*e, n))).map(V::Elem).collect())
            }
            ("app.repository", "getRelationshipsOf") => {
                let V::Elem(target) = a(0) else { return Ok(arr(vec![])) };
                let filter = a(1);
                let mut out = vec![];
                for x in repo.all().collect::<Vec<_>>() {
                    if !repo.kind_of(x, "Relationship") {
                        continue;
                    }
                    let ends = ["source", "target"].iter().filter_map(|f| repo.refd(x, f)).chain(["end1", "end2"].iter().filter_map(|f| repo.child(x, f)).filter_map(|end| repo.refd(end, "reference"))).any(|o| o == target);
                    if ends && (matches!(filter, V::Undef) || truthy(&self.call(&filter, V::Undef, vec![V::Elem(x)])?)) {
                        out.push(V::Elem(x));
                    }
                }
                arr(out)
            }
            ("app.repository", "getViewsOf") => {
                let V::Elem(m) = a(0) else { return Ok(arr(vec![])) };
                arr(repo.all().filter(|v| repo.refd(*v, "model") == Some(m)).map(V::Elem).collect())
            }
            ("app.project", "getProject") => V::Elem(repo.root),
            ("app.project", "getFilename") => s(&self.filename),
            ("_", _) => self.lodash(m, args)?,
            _ => return type_err(format!("{h}.{m} is not a function")),
        })
    }

    fn lodash(&mut self, m: &str, args: Vec<V>) -> R<V> {
        let a = |i: usize| args.get(i).cloned().unwrap_or(V::Undef);
        let coll = |me: &mut Self| -> R<Vec<V>> {
            match a(0) {
                V::Undef | V::Null => Ok(vec![]),
                v @ V::Arr(_) => me.iterate(&v),
                v => {
                    let mut out = vec![];
                    for k in me.keys(&v) {
                        out.push(me.get(&v, &k)?);
                    }
                    Ok(out)
                }
            }
        };
        // an iteratee: a function, or a property name
        let key = |me: &mut Self, f: &V, x: &V| -> R<V> {
            match f {
                V::Func(_) => me.call(f, V::Undef, vec![x.clone()]),
                V::Undef => Ok(x.clone()),
                k => {
                    let k = to_str(k);
                    match x {
                        V::Undef | V::Null => Ok(V::Undef),
                        _ => me.get(x, &k),
                    }
                }
            }
        };
        Ok(match m {
            "sortBy" => {
                let items = coll(self)?;
                let f = a(1);
                let mut keyed = vec![];
                for x in items {
                    let k = key(self, &f, &x)?;
                    keyed.push(arr(vec![k, x]));
                }
                let sorted = self.sort_keyed(keyed)?;
                arr(sorted)
            }
            "filter" | "map" | "find" | "each" | "forEach" | "some" | "every" => {
                let items = coll(self)?;
                let f = a(1);
                let mut out = vec![];
                for x in items {
                    let r = key(self, &f, &x)?;
                    match m {
                        "map" => out.push(r),
                        "filter" if truthy(&r) => out.push(x),
                        "find" if truthy(&r) => return Ok(x),
                        "some" if truthy(&r) => return Ok(V::Bool(true)),
                        "every" if !truthy(&r) => return Ok(V::Bool(false)),
                        _ => {}
                    }
                }
                match m {
                    "map" | "filter" => arr(out),
                    "some" => V::Bool(false),
                    "every" => V::Bool(true),
                    "each" | "forEach" => a(0),
                    _ => V::Undef,
                }
            }
            "groupBy" => {
                let items = coll(self)?;
                let f = a(1);
                let mut out: IndexMap<String, V> = IndexMap::new();
                for x in items {
                    let k = key(self, &f, &x)?;
                    let k = self.string(&k)?;
                    match out.get(&k) {
                        Some(V::Arr(v)) => v.borrow_mut().push(x),
                        _ => {
                            out.insert(k, arr(vec![x]));
                        }
                    }
                }
                obj(out)
            }
            "uniq" => {
                let mut out: Vec<V> = vec![];
                for x in coll(self)? {
                    if !out.iter().any(|y| same_value_zero(y, &x)) {
                        out.push(x);
                    }
                }
                arr(out)
            }
            "flatten" => {
                let mut out = vec![];
                for x in coll(self)? {
                    match x {
                        V::Arr(i) => out.extend(i.borrow().iter().cloned()),
                        x => out.push(x),
                    }
                }
                arr(out)
            }
            "compact" => arr(coll(self)?.into_iter().filter(truthy).collect()),
            "includes" | "contains" => {
                let x = a(1);
                match a(0) {
                    V::Str(t) => V::Bool(t.contains(&*to_str(&x))),
                    _ => V::Bool(coll(self)?.iter().any(|y| same_value_zero(y, &x))),
                }
            }
            "keys" => arr(self.keys(&a(0)).into_iter().map(|k| s(&k)).collect()),
            "values" => arr(coll(self)?),
            "isString" => V::Bool(matches!(a(0), V::Str(_))),
            "isArray" => V::Bool(matches!(a(0), V::Arr(_))),
            "isNumber" => V::Bool(matches!(a(0), V::Num(_))),
            "isObject" => V::Bool(matches!(a(0), V::Arr(_) | V::Obj(_) | V::Func(_) | V::Elem(_) | V::Regex(_))),
            "isFunction" => V::Bool(matches!(a(0), V::Func(_))),
            "isEmpty" => V::Bool(match a(0) {
                V::Arr(x) => x.borrow().is_empty(),
                V::Obj(o) => o.borrow().is_empty(),
                V::Str(t) => t.is_empty(),
                V::Elem(_) => false,
                _ => true,
            }),
            "first" | "head" => coll(self)?.into_iter().next().unwrap_or(V::Undef),
            "last" => coll(self)?.pop().unwrap_or(V::Undef),
            "capitalize" => {
                let t = self.string(&a(0))?.to_lowercase();
                let mut c = t.chars();
                s(&c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default())
            }
            "upperFirst" => {
                let t = self.string(&a(0))?;
                let mut c = t.chars();
                s(&c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default())
            }
            "escape" => s(&escape_xml(&self.string(&a(0))?).replace("&#34;", "&quot;")),
            _ => return type_err(format!("_.{m} is not a function in UMLCraft templates")),
        })
    }

    /// Sort [key, value] pairs by key (numbers numerically, else strings), stable; returns the values.
    fn sort_keyed(&mut self, pairs: Vec<V>) -> R<Vec<V>> {
        let mut keyed: Vec<(V, V)> = vec![];
        for p in pairs {
            let k = self.get(&p, "0")?;
            let v = self.get(&p, "1")?;
            keyed.push((k, v));
        }
        let mut strs = vec![];
        for (k, _) in &keyed {
            strs.push(self.string(k)?);
        }
        let mut idx: Vec<usize> = (0..keyed.len()).collect();
        idx.sort_by(|&i, &j| {
            let (a, b) = (&keyed[i].0, &keyed[j].0);
            match (a, b) {
                (V::Undef, V::Undef) => std::cmp::Ordering::Equal,
                (V::Undef, _) => std::cmp::Ordering::Greater,
                (_, V::Undef) => std::cmp::Ordering::Less,
                (V::Num(x), V::Num(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                _ => strs[i].cmp(&strs[j]),
            }
        });
        Ok(idx.into_iter().map(|i| keyed[i].1.clone()).collect())
    }

    /// A template value as JSON (elements as they are written in the file). None for undefined and
    /// functions.
    fn to_json(&mut self, v: &V, depth: usize) -> R<Option<Json>> {
        if depth > 200 {
            return type_err("Converting circular structure to JSON");
        }
        Ok(Some(match v {
            V::Undef | V::Func(_) | V::Type(_) | V::Host(_) => return Ok(None),
            V::Null => Json::Null,
            V::Bool(b) => Json::Bool(*b),
            V::Num(n) => {
                if n.is_finite() {
                    crate::model::num(*n)
                } else {
                    Json::Null
                }
            }
            V::Str(x) => Json::String(x.to_string()),
            V::Arr(a) => {
                let items = a.borrow().clone();
                let mut out = vec![];
                for x in items {
                    out.push(self.to_json(&x, depth + 1)?.unwrap_or(Json::Null));
                }
                Json::Array(out)
            }
            V::Obj(o) => {
                let items: Vec<(String, V)> = o.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let mut m = serde_json::Map::new();
                for (k, x) in items {
                    if let Some(j) = self.to_json(&x, depth + 1)? {
                        m.insert(k, j);
                    }
                }
                Json::Object(m)
            }
            V::Elem(e) => self.repo.export(*e),
            V::Regex(_) => Json::Object(serde_json::Map::new()),
        }))
    }

    /// JSON as plain values (a `$ref` stays an object).
    fn json_plain(&self, j: &Json) -> V {
        match j {
            Json::Null => V::Null,
            Json::Bool(b) => V::Bool(*b),
            Json::Number(n) => V::Num(n.as_f64().unwrap_or(0.0)),
            Json::String(x) => s(x),
            Json::Array(a) => arr(a.iter().map(|x| self.json_plain(x)).collect()),
            Json::Object(m) => obj(m.iter().map(|(k, x)| (k.clone(), self.json_plain(x))).collect()),
        }
    }
}

const HOSTS: &[&str] = &["Math", "JSON", "Object", "Array", "Number", "String", "Boolean", "console", "app", "app.repository", "app.project", "type", "_"];

fn host_get(h: &'static str, k: &str) -> V {
    match (h, k) {
        ("Math", "PI") => V::Num(std::f64::consts::PI),
        ("Math", "E") => V::Num(std::f64::consts::E),
        ("Math", "LN2") => V::Num(std::f64::consts::LN_2),
        ("Math", "SQRT2") => V::Num(std::f64::consts::SQRT_2),
        ("Number", "MAX_SAFE_INTEGER") => V::Num(9007199254740991.0),
        ("Number", "MIN_SAFE_INTEGER") => V::Num(-9007199254740991.0),
        ("Number", "MAX_VALUE") => V::Num(f64::MAX),
        ("Number", "EPSILON") => V::Num(f64::EPSILON),
        ("Number", "POSITIVE_INFINITY") => V::Num(f64::INFINITY),
        ("Number", "NEGATIVE_INFINITY") => V::Num(f64::NEG_INFINITY),
        ("Number", "NaN") => V::Num(f64::NAN),
        ("app", "repository") => V::Host("app.repository"),
        ("app", "project") => V::Host("app.project"),
        ("app", _) => V::Undef,
        ("type", _) => V::Type(k.into()),
        (_, "name") => s(h),
        _ => method(&V::Host(h), k),
    }
}

const ARRAY_METHODS: &[&str] = &[
    "push", "pop", "shift", "unshift", "forEach", "map", "filter", "find", "findIndex", "findLast", "findLastIndex", "some", "every", "flatMap", "reduce", "reduceRight", "join", "toString", "indexOf", "lastIndexOf", "includes", "slice",
    "splice", "concat", "reverse", "sort", "flat", "at", "fill", "keys", "entries", "values",
];

const STRING_METHODS: &[&str] = &[
    "toUpperCase", "toLowerCase", "toLocaleUpperCase", "toLocaleLowerCase", "trim", "trimStart", "trimEnd", "trimLeft", "trimRight", "toString", "valueOf", "charAt", "charCodeAt", "codePointAt", "at", "indexOf", "lastIndexOf", "includes",
    "startsWith", "endsWith", "slice", "substring", "substr", "split", "replace", "replaceAll", "match", "matchAll", "search", "padStart", "padEnd", "repeat", "concat", "localeCompare", "normalize",
];

const ELEMENT_METHODS: &[&str] = &[
    "getClassName", "getDisplayClassName", "getPathname", "getNodeText", "getParentField", "traverse", "traverseDepthFirst", "traverseField", "traverseFieldDepthFirst", "traverseUp", "getTag", "getTagValue", "getGeneralElements",
    "getSpecialElements", "getReturnParameter", "getNonReturnParameters", "getVisibilityString", "getStereotypeString",
];

/// The fields that hold lists in StarUML's metamodels ("objs" and "refs" kinds, metamodel.json of each
/// notation): empty when the file leaves them out.
const COLLECTION_FIELDS: &[&str] = &[
    "attributes", "behaviors", "bodyConditions", "columns", "connections", "constraints", "doActivities", "edges", "effects", "entryActivities", "eventDefinitions", "exitActivities", "extensionPoints", "flowProperties", "fragments",
    "groups", "inputs", "literals", "localPostconditions", "localPreconditions", "messages", "nodes", "operands", "operations", "outputs", "ownedElements", "ownedViews", "parameterSubstitutions", "parameters", "participants", "parts",
    "ports", "postconditions", "preconditions", "properties", "qualifiers", "receptions", "references", "regions", "slots", "subViews", "subgroups", "tags", "templateParameters", "transitions", "triggers", "values", "vertices",
    "constrainedElements", "containedViews", "conveyed", "exceptionTypes", "extensionLocations", "importedElements", "inStates", "lowerParticipants", "raisedExceptions", "realizingActivityEdges", "realizingConnectors",
    "realizingMessages", "represented", "stakeholders", "upperParticipants",
];

/// Reference and embedded-object fields ("ref", "var" and "obj" kinds): null when left out.
const REFERENCE_FIELDS: &[&str] = &[
    "model", "head", "tail", "source", "target", "type", "stereotype", "reference", "classifier", "defaultValue", "end1", "end2", "icon", "containerView", "hostEdge", "hostNode", "association", "signal", "operation", "covered",
    "signature", "connector", "activation", "referenceTo", "represent", "handlerBody", "submachine", "subactivity", "definingFeature", "messageRef", "linePart", "parameter", "targetOperation", "targetSignal",
];

pub fn truthy(v: &V) -> bool {
    match v {
        V::Undef | V::Null => false,
        V::Bool(b) => *b,
        V::Num(n) => *n != 0.0 && !n.is_nan(),
        V::Str(x) => !x.is_empty(),
        _ => true,
    }
}

pub fn type_of(v: &V) -> &'static str {
    match v {
        V::Undef => "undefined",
        V::Null | V::Arr(_) | V::Obj(_) | V::Elem(_) | V::Regex(_) => "object",
        V::Bool(_) => "boolean",
        V::Num(_) => "number",
        V::Str(_) => "string",
        V::Func(_) | V::Type(_) => "function",
        V::Host(h) => {
            if matches!(*h, "String" | "Number" | "Boolean" | "Array" | "Object") {
                "function"
            } else {
                "object"
            }
        }
    }
}

pub fn strict_eq(a: &V, b: &V) -> bool {
    match (a, b) {
        (V::Undef, V::Undef) | (V::Null, V::Null) => true,
        (V::Bool(x), V::Bool(y)) => x == y,
        (V::Num(x), V::Num(y)) => x == y,
        (V::Str(x), V::Str(y)) => x == y,
        (V::Arr(x), V::Arr(y)) => Rc::ptr_eq(x, y),
        (V::Obj(x), V::Obj(y)) => Rc::ptr_eq(x, y),
        (V::Func(x), V::Func(y)) => Rc::ptr_eq(x, y),
        (V::Elem(x), V::Elem(y)) => x == y,
        (V::Type(x), V::Type(y)) => x == y,
        (V::Host(x), V::Host(y)) => x == y,
        (V::Regex(x), V::Regex(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

fn same_value_zero(a: &V, b: &V) -> bool {
    match (a, b) {
        (V::Num(x), V::Num(y)) if x.is_nan() && y.is_nan() => true,
        _ => strict_eq(a, b),
    }
}

/// ToString for primitives (objects: "[object Object]"; use Interp::string for arrays).
pub fn to_str(v: &V) -> String {
    match v {
        V::Undef => "undefined".into(),
        V::Null => "null".into(),
        V::Bool(b) => b.to_string(),
        V::Num(n) => num_str(*n),
        V::Str(x) => x.to_string(),
        V::Arr(a) => a.borrow().iter().map(|x| if matches!(x, V::Undef | V::Null) { String::new() } else { to_str(x) }).collect::<Vec<_>>().join(","),
        V::Obj(_) | V::Elem(_) => "[object Object]".into(),
        V::Func(_) => "function () { [native code] }".into(),
        V::Type(t) => format!("class {t} {{ [native code] }}"),
        V::Host(h) => format!("[object {h}]"),
        V::Regex(r) => format!("/{}/{}", r.source, r.flags),
    }
}

/// A short description of a value for error messages.
fn short(v: &V) -> String {
    match v {
        V::Elem(_) => "element".into(),
        V::Host(h) => h.to_string(),
        V::Str(x) => format!("\"{x}\""),
        _ => to_str(v),
    }
}

pub fn to_num(v: &V) -> f64 {
    match v {
        V::Undef => f64::NAN,
        V::Null => 0.0,
        V::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        V::Num(n) => *n,
        V::Str(x) => {
            let t = x.trim();
            if t.is_empty() {
                0.0
            } else if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                i64::from_str_radix(h, 16).map(|n| n as f64).unwrap_or(f64::NAN)
            } else if t == "Infinity" || t == "+Infinity" {
                f64::INFINITY
            } else if t == "-Infinity" {
                f64::NEG_INFINITY
            } else if t.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-')) {
                t.parse().unwrap_or(f64::NAN)
            } else {
                f64::NAN
            }
        }
        V::Arr(a) => {
            let t = to_str(&V::Arr(a.clone()));
            to_num(&s(&t))
        }
        _ => f64::NAN,
    }
}

fn to_i32(n: f64) -> i32 {
    if !n.is_finite() {
        0
    } else {
        (n.trunc() as i64 as u64 & 0xffff_ffff) as u32 as i32
    }
}

fn radix_str(mut n: i64, radix: u32) -> String {
    let radix = radix.clamp(2, 36) as i64;
    if n == 0 {
        return "0".into();
    }
    let neg = n < 0;
    let mut out = vec![];
    while n != 0 {
        let d = (n % radix).unsigned_abs() as u32;
        out.push(std::char::from_digit(d, radix as u32).unwrap_or('0'));
        n /= radix;
    }
    if neg {
        out.push('-');
    }
    out.iter().rev().collect()
}

/// EJS's escapeXML: & < > " ' (ejs/lib/utils.js:49-55).
pub fn escape_xml(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&#34;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// The message of an uncaught error, as `err.toString()` writes it.
pub fn thrown_message(it: &mut Interp, t: &Thrown) -> String {
    match &t.0 {
        V::Obj(o) => {
            let (n, m) = {
                let b = o.borrow();
                (b.get("name").map(to_str), b.get("message").map(to_str))
            };
            match (n, m) {
                (Some(n), Some(m)) => {
                    if m.is_empty() {
                        n
                    } else {
                        format!("{n}: {m}")
                    }
                }
                _ => it.string(&t.0).unwrap_or_default(),
            }
        }
        v => it.string(v).unwrap_or_default(),
    }
}
