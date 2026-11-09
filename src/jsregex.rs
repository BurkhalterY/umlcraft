//! JavaScript regular expressions for templates (src/js.rs): a backtracking matcher over characters.
//! Supported: literals, `.`, classes (`[a-z]`, `[^…]`, `\d \w \s` and their negations), anchors
//! `^ $`, word boundaries `\b \B`, groups (capturing, `(?:…)`, named `(?<n>…)` captured by number),
//! lookahead `(?=…)` and `(?!…)`, back references `\1`…, alternation, the quantifiers `* + ? {n}
//! {n,} {n,m}` (greedy or lazy) and the flags g, i, m, s and y (u is accepted). Not supported:
//! lookbehind and Unicode property escapes.

use std::cell::Cell;

#[derive(Debug, Clone)]
enum Item {
    Ch(char),
    Range(char, char),
    Digit(bool),
    Word(bool),
    Space(bool),
}

#[derive(Debug, Clone)]
enum Node {
    Char(char),
    Any,
    Class(Vec<Item>, bool),
    Start,
    End,
    WordB(bool),
    Group(Box<Node>, Option<usize>),
    Look(Box<Node>, bool),
    Backref(usize),
    Concat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat(Box<Node>, usize, usize, bool),
}

pub struct Regex {
    pub source: String,
    pub flags: String,
    pub global: bool,
    pub sticky: bool,
    pub last_index: Cell<usize>,
    icase: bool,
    multiline: bool,
    dot_all: bool,
    groups: usize,
    root: Node,
}

/// A match: its range and each group's range (group 0 is the whole match), in characters.
#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub groups: Vec<Option<(usize, usize)>>,
}

struct P<'a> {
    s: &'a [char],
    i: usize,
    groups: usize,
}

impl<'a> P<'a> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn alt(&mut self) -> Result<Node, String> {
        let mut alts = vec![self.concat()?];
        while self.peek() == Some('|') {
            self.i += 1;
            alts.push(self.concat()?);
        }
        Ok(if alts.len() == 1 { alts.pop().unwrap() } else { Node::Alt(alts) })
    }

    fn concat(&mut self) -> Result<Node, String> {
        let mut items = vec![];
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.atom()?;
            let atom = self.quantified(atom)?;
            items.push(atom);
        }
        Ok(Node::Concat(items))
    }

    fn number(&mut self) -> Option<usize> {
        let st = self.i;
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.i += 1;
        }
        self.s[st..self.i].iter().collect::<String>().parse().ok()
    }

    fn quantified(&mut self, atom: Node) -> Result<Node, String> {
        let (min, max) = match self.peek() {
            Some('*') => (0, usize::MAX),
            Some('+') => (1, usize::MAX),
            Some('?') => (0, 1),
            Some('{') => {
                let save = self.i;
                self.i += 1;
                match self.number() {
                    Some(n) => {
                        let m = if self.peek() == Some(',') {
                            self.i += 1;
                            if self.peek() == Some('}') {
                                usize::MAX
                            } else {
                                self.number().ok_or("Incomplete quantifier")?
                            }
                        } else {
                            n
                        };
                        if self.peek() != Some('}') {
                            self.i = save;
                            return Ok(atom);
                        }
                        (n, m)
                    }
                    None => {
                        // a literal `{`
                        self.i = save;
                        return Ok(atom);
                    }
                }
            }
            _ => return Ok(atom),
        };
        self.i += 1;
        if matches!(atom, Node::Start | Node::End | Node::WordB(_) | Node::Look(..)) {
            return Err("Nothing to repeat".into());
        }
        let greedy = if self.peek() == Some('?') {
            self.i += 1;
            false
        } else {
            true
        };
        Ok(Node::Repeat(Box::new(atom), min, max, greedy))
    }

    fn escape_char(&mut self, c: char) -> Result<char, String> {
        let hex = |p: &mut Self, n: usize| -> Result<u32, String> {
            let mut v = 0;
            for _ in 0..n {
                let d = p.peek().and_then(|c| c.to_digit(16)).ok_or("Invalid escape")?;
                v = v * 16 + d;
                p.i += 1;
            }
            Ok(v)
        };
        Ok(match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            'f' => '\u{c}',
            'v' => '\u{b}',
            '0' => '\0',
            'x' => char::from_u32(hex(self, 2)?).unwrap_or('\u{fffd}'),
            'u' => {
                if self.peek() == Some('{') {
                    self.i += 1;
                    let mut v = 0;
                    while let Some(d) = self.peek().and_then(|c| c.to_digit(16)) {
                        v = v * 16 + d;
                        self.i += 1;
                    }
                    self.i += 1;
                    char::from_u32(v).unwrap_or('\u{fffd}')
                } else {
                    char::from_u32(hex(self, 4)?).unwrap_or('\u{fffd}')
                }
            }
            c => c,
        })
    }

    fn atom(&mut self) -> Result<Node, String> {
        let c = self.peek().ok_or("Unexpected end")?;
        self.i += 1;
        Ok(match c {
            '.' => Node::Any,
            '^' => Node::Start,
            '$' => Node::End,
            '*' | '+' | '?' => return Err("Nothing to repeat".into()),
            '(' => {
                let node = if self.s[self.i..].starts_with(&['?', ':']) {
                    self.i += 2;
                    let n = self.alt()?;
                    Node::Group(Box::new(n), None)
                } else if self.s[self.i..].starts_with(&['?', '=']) || self.s[self.i..].starts_with(&['?', '!']) {
                    let neg = self.s[self.i + 1] == '!';
                    self.i += 2;
                    let n = self.alt()?;
                    Node::Look(Box::new(n), neg)
                } else if self.s[self.i..].starts_with(&['?', '<']) && !matches!(self.s.get(self.i + 2), Some('=' | '!')) {
                    while self.peek().map(|c| c != '>').unwrap_or(false) {
                        self.i += 1;
                    }
                    self.i += 1;
                    self.groups += 1;
                    let g = self.groups;
                    let n = self.alt()?;
                    Node::Group(Box::new(n), Some(g))
                } else if self.peek() == Some('?') {
                    return Err("Lookbehind is not supported".into());
                } else {
                    self.groups += 1;
                    let g = self.groups;
                    let n = self.alt()?;
                    Node::Group(Box::new(n), Some(g))
                };
                if self.peek() != Some(')') {
                    return Err("Unterminated group".into());
                }
                self.i += 1;
                node
            }
            ')' => return Err("Unmatched ')'".into()),
            '[' => {
                let neg = self.peek() == Some('^');
                if neg {
                    self.i += 1;
                }
                let mut items = vec![];
                let mut first = true;
                loop {
                    let Some(c) = self.peek() else { return Err("Unterminated character class".into()) };
                    self.i += 1;
                    if c == ']' && !first {
                        break;
                    }
                    if c == ']' {
                        // `[]` matches nothing; `[^]` anything
                        break;
                    }
                    first = false;
                    let item = if c == '\\' {
                        let e = self.peek().ok_or("Unterminated character class")?;
                        self.i += 1;
                        match e {
                            'd' => Item::Digit(true),
                            'D' => Item::Digit(false),
                            'w' => Item::Word(true),
                            'W' => Item::Word(false),
                            's' => Item::Space(true),
                            'S' => Item::Space(false),
                            'b' => Item::Ch('\u{8}'),
                            e => Item::Ch(self.escape_char(e)?),
                        }
                    } else {
                        Item::Ch(c)
                    };
                    // a range a-z
                    if let Item::Ch(lo) = item {
                        if self.peek() == Some('-') && self.s.get(self.i + 1).map(|c| *c != ']').unwrap_or(false) {
                            self.i += 1;
                            let mut hi = self.peek().ok_or("Unterminated character class")?;
                            self.i += 1;
                            if hi == '\\' {
                                let e = self.peek().ok_or("Unterminated character class")?;
                                self.i += 1;
                                hi = self.escape_char(e)?;
                            }
                            if hi < lo {
                                return Err("Range out of order in character class".into());
                            }
                            items.push(Item::Range(lo, hi));
                            continue;
                        }
                    }
                    items.push(item);
                }
                Node::Class(items, neg)
            }
            '\\' => {
                let e = self.peek().ok_or("\\ at end of pattern")?;
                self.i += 1;
                match e {
                    'd' => Node::Class(vec![Item::Digit(true)], false),
                    'D' => Node::Class(vec![Item::Digit(true)], true),
                    'w' => Node::Class(vec![Item::Word(true)], false),
                    'W' => Node::Class(vec![Item::Word(true)], true),
                    's' => Node::Class(vec![Item::Space(true)], false),
                    'S' => Node::Class(vec![Item::Space(true)], true),
                    'b' => Node::WordB(true),
                    'B' => Node::WordB(false),
                    '1'..='9' => {
                        self.i -= 1;
                        Node::Backref(self.number().unwrap_or(0))
                    }
                    'p' | 'P' if self.peek() == Some('{') => return Err("Unicode property escapes are not supported".into()),
                    e => Node::Char(self.escape_char(e)?),
                }
            }
            c => Node::Char(c),
        })
    }
}

type Caps = Vec<Option<(usize, usize)>>;

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

impl Regex {
    pub fn new(source: &str, flags: &str) -> Result<Regex, String> {
        for f in flags.chars() {
            if !"gimsuyd".contains(f) || flags.matches(f).count() > 1 {
                return Err(format!("Invalid flags '{flags}'"));
            }
        }
        let chars: Vec<char> = source.chars().collect();
        let mut p = P { s: &chars, i: 0, groups: 0 };
        let root = p.alt()?;
        if p.i < chars.len() {
            return Err("Unmatched ')'".into());
        }
        Ok(Regex {
            source: source.to_string(),
            flags: flags.to_string(),
            global: flags.contains('g'),
            sticky: flags.contains('y'),
            last_index: Cell::new(0),
            icase: flags.contains('i'),
            multiline: flags.contains('m'),
            dot_all: flags.contains('s'),
            groups: p.groups,
            root,
        })
    }

    /// The first match at or after `from` (only at `from` for a sticky regex).
    pub fn find_at(&self, s: &[char], from: usize) -> Option<Match> {
        if self.sticky {
            return self.match_at(s, from);
        }
        (from..=s.len()).find_map(|i| self.match_at(s, i))
    }

    /// A match starting exactly at `pos`.
    pub fn match_at(&self, s: &[char], pos: usize) -> Option<Match> {
        if pos > s.len() {
            return None;
        }
        let mut caps: Caps = vec![None; self.groups + 1];
        let mut end = None;
        if self.m(&self.root, s, pos, &mut caps, &mut |p, _| {
            end = Some(p);
            true
        }) {
            let e = end?;
            caps[0] = Some((pos, e));
            // the captures of the successful path were kept by m
            return Some(Match { start: pos, end: e, groups: caps });
        }
        None
    }

    fn eq(&self, a: char, b: char) -> bool {
        a == b || (self.icase && a.to_lowercase().eq(b.to_lowercase()))
    }

    fn class_has(&self, items: &[Item], c: char) -> bool {
        let one = |c: char| {
            items.iter().any(|it| match it {
                Item::Ch(x) => *x == c,
                Item::Range(a, b) => *a <= c && c <= *b,
                Item::Digit(p) => c.is_ascii_digit() == *p,
                Item::Word(p) => is_word(c) == *p,
                Item::Space(p) => is_space(c) == *p,
            })
        };
        one(c) || (self.icase && (c.to_lowercase().any(one) || c.to_uppercase().any(one)))
    }

    /// Match a node at `pos`, then the continuation; captures found on the way are kept only when the
    /// whole match succeeds.
    fn m(&self, n: &Node, s: &[char], pos: usize, caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool) -> bool {
        match n {
            Node::Char(c) => pos < s.len() && self.eq(s[pos], *c) && k(pos + 1, caps),
            Node::Any => pos < s.len() && (self.dot_all || !matches!(s[pos], '\n' | '\r' | '\u{2028}' | '\u{2029}')) && k(pos + 1, caps),
            Node::Class(items, neg) => pos < s.len() && self.class_has(items, s[pos]) != *neg && k(pos + 1, caps),
            Node::Start => (pos == 0 || (self.multiline && matches!(s[pos - 1], '\n' | '\r'))) && k(pos, caps),
            Node::End => (pos == s.len() || (self.multiline && matches!(s[pos], '\n' | '\r'))) && k(pos, caps),
            Node::WordB(want) => {
                let a = pos > 0 && is_word(s[pos - 1]);
                let b = pos < s.len() && is_word(s[pos]);
                ((a != b) == *want) && k(pos, caps)
            }
            Node::Group(inner, idx) => match idx {
                None => self.m(inner, s, pos, caps, k),
                Some(i) => {
                    let i = *i;
                    self.m(inner, s, pos, caps, &mut |p, c: &mut Caps| {
                        let old = c[i];
                        c[i] = Some((pos, p));
                        if k(p, c) {
                            true
                        } else {
                            c[i] = old;
                            false
                        }
                    })
                }
            },
            Node::Look(inner, neg) => {
                let mut trial = caps.clone();
                let found = self.m(inner, s, pos, &mut trial, &mut |_, _| true);
                if *neg {
                    !found && k(pos, caps)
                } else if found {
                    let saved = caps.clone();
                    *caps = trial;
                    if k(pos, caps) {
                        true
                    } else {
                        *caps = saved;
                        false
                    }
                } else {
                    false
                }
            }
            Node::Backref(i) => {
                let Some(Some((a, b))) = caps.get(*i).copied() else { return k(pos, caps) };
                let len = b - a;
                if pos + len > s.len() {
                    return false;
                }
                (0..len).all(|j| self.eq(s[pos + j], s[a + j])) && k(pos + len, caps)
            }
            Node::Concat(items) => self.seq(items, 0, s, pos, caps, k),
            Node::Alt(alts) => alts.iter().any(|a| self.m(a, s, pos, caps, k)),
            Node::Repeat(inner, min, max, greedy) => self.rep(inner, *min, *max, *greedy, 0, s, pos, caps, k),
        }
    }

    fn seq(&self, items: &[Node], i: usize, s: &[char], pos: usize, caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool) -> bool {
        if i == items.len() {
            return k(pos, caps);
        }
        self.m(&items[i], s, pos, caps, &mut |p, c: &mut Caps| self.seq(items, i + 1, s, p, c, k))
    }

    #[allow(clippy::too_many_arguments)]
    fn rep(&self, n: &Node, min: usize, max: usize, greedy: bool, count: usize, s: &[char], pos: usize, caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool) -> bool {
        let more = |c: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool| -> bool {
            count < max
                && self.m(n, s, pos, c, &mut |p, c2: &mut Caps| {
                    // an empty iteration past the minimum ends the loop
                    if p == pos && count >= min {
                        return false;
                    }
                    self.rep(n, min, max, greedy, count + 1, s, p, c2, k)
                })
        };
        if greedy {
            more(caps, k) || (count >= min && k(pos, caps))
        } else {
            (count >= min && k(pos, caps)) || more(caps, k)
        }
    }
}
