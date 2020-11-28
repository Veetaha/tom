//! FIXME: write short doc here

use drop_bomb::DebugDropBomb;
use crate::{
    parser::Parser,
    SyntaxKind::{self, *},
    T,
};

struct Mark {
    kind: SyntaxKind,
    bomb: DebugDropBomb,
}

impl Mark {
    fn new(kind: SyntaxKind) -> Mark {
        Mark {
            kind,
            bomb: DebugDropBomb::new("Mark dropped"),
        }
    }
}

impl<'t, 's> Parser<'t, 's> {
    fn start(&mut self, s: SyntaxKind) -> Mark {
        self.sink.start(s);
        Mark::new(s)
    }

    fn finish(&mut self, mut m: Mark) {
        m.bomb.defuse();
        self.sink.finish(m.kind);
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.sink.error(msg);
    }

    fn at(&self, lookahead: usize) -> SyntaxKind {
        let pos = self.pos + lookahead;
        if pos >= self.tokens.len() {
            return EOF;
        }
        let pos = self.tokens[pos];
        self.tokens[pos].kind
    }

    fn current(&self) -> SyntaxKind {
        self.at(0)
    }

    fn eat(&mut self, s: SyntaxKind) {
        if self.current() == s {
            self.bump()
        } else {
            self.bump_error(format!("expected {:?}", s))
        }
    }

    fn bump(&mut self) {
        if self.pos == self.tokens.significant.len() {
            panic!("bumping past EOF");
        }
        let pos = self.tokens.significant[self.pos];
        self.sink.token(pos, None);
        self.pos += 1;
    }

    fn bump_remap(&mut self, s: SyntaxKind) {
        if self.pos == self.tokens.significant.len() {
            panic!("bumping past EOF");
        }
        let pos = self.tokens.significant[self.pos];
        self.sink.token(pos, Some(s));
        self.pos += 1;
    }

    fn bump_error(&mut self, msg: impl Into<String>) {
        match self.current() {
            EOF => {
                self.error(msg);
            }
            _ => {
                let m = self.start(ERROR);
                self.error(msg);
                self.bump();
                self.finish(m);
            }
        }
    }
}

impl<'s, 't> Parser<'s, 't> {
    pub(crate) fn parse(&mut self) {
        self.doc();
    }

    // test-doc
    // key = "value"
    //
    // [table]
    // a = 2
    //
    // [[array-table]]
    // b = 3
    fn doc(&mut self) {
        let m = self.start(DOC);
        self.entries();
        while self.current() != EOF {
            match self.current() {
                T!['['] => {
                    if self.at(1) == T!['['] {
                        self.array_table()
                    } else {
                        self.table()
                    }
                }
                _ => self.bump_error("expected `[`"),
            }
        }

        self.finish(m);
    }

    // test-entries
    // foo = 92
    // 'bar' = 14
    fn entries(&mut self) {
        while !matches!(self.current(), EOF | T!['[']) {
            match self.current() {
                BARE_KEY_LIKE | BASIC_LINE_STRING | LITERAL_LINE_STRING => self.entry(),
                _ => self.bump_error("expected a key"),
            }
        }
    }

    fn entry(&mut self) {
        let m = self.start(ENTRY);
        self.keys();
        self.eat(T![=]);
        self.val();
        self.finish(m);
    }

    // test-keys
    // foo = 1
    // foo.bar = 2
    // 3.14 = 4.13
    // 2020-29-06 = 2020-29-06
    fn keys(&mut self) {
        // []
        // [foo]
        // [foo.bar]
        // [foo.]
        let mut first = true;
        while !matches!(self.current(), EOF | T![']'] | T![=]) {
            if !first {
                self.eat(T![.]);
            }
            first = false;
            self.key();
        }
    }

    fn key(&mut self) {
        let m = self.start(KEY);
        match self.current() {
            // test-key
            // foo = 92
            // 92 = 92
            // 1914-08-26 = 92
            BARE_KEY_LIKE /* TODO: remove | BARE_KEY_OR_NUMBER | BARE_KEY_OR_DATE */ => self.bump_remap(BARE_KEY),
            // test-key-str
            // "foo" = 92
            // 'bar' = 92
            BASIC_LINE_STRING | LITERAL_LINE_STRING => self.bump(),
            _ => self.bump_error("expected a key"),
        }
        self.finish(m);
    }

    fn val(&mut self) {
        let m = self.start(VALUE);
        match self.current() {
            // test-val-num
            // a = 92
            // b = 8.5
            PLUS | BARE_KEY_LIKE => {
                // test-date-time
                // a = 1914-08-26
                // b = 1979-05-27T07:32:00-08:00

                self.bump_remap(NUMBER)
            }
            // test-val-bool
            // a = true
            // b = false
            TRUE | FALSE => self.bump(),
            // test-val-str
            // a = "hello\nworld"
            // b = """
            //   hello
            //   world
            // """
            BASIC_STRING | MULTILINE_BASIC_STRING => self.bump(),
            // test-val-lit
            // a = 'hello\nworld'
            // b = '''
            //   hello
            //   world
            // '''
            LITERAL_STRING | MULTILINE_LITERAL_STRING => self.bump(),
            // test-val-array
            // a = [1, "foo"]
            T!['['] => self.array(),
            // test-val-inline
            // a = { "foo" = 1, bar = 2, }
            L_CURLY => self.dict(),
            // test-val-unexpected
            // foo = _
            _ => self.bump_error("expected a value"),
        }
        self.finish(m);
    }

    fn array(&mut self) {
        assert_eq!(self.current(), T!['[']);
        let m = self.start(ARRAY);
        self.bump();
        while self.current() != EOF && self.current() != R_BRACK {
            self.val();
            // test-array
            // a = []
            // b = [1]
            // c = [1,]
            // d = [,]
            // e = [1 1]
            if self.current() != R_BRACK {
                self.eat(COMMA)
            }
        }
        self.eat(R_BRACK);

        self.finish(m);
    }

    fn dict(&mut self) {
        assert_eq!(self.current(), L_CURLY);
        let m = self.start(DICT);
        self.bump();
        while self.current() != EOF && self.current() != R_CURLY {
            // test-inline-key
            // a = { dotted.key = 92 }
            self.entry();
            // test-inline
            // a = {}
            // b = {foo=1}
            // c = {foo=1,}
            // d = {,}
            if self.current() != R_CURLY {
                self.eat(COMMA)
            }
        }
        self.eat(R_CURLY);

        self.finish(m);
    }

    // test-table
    // [table]
    // a = 1
    // b = 2
    fn table(&mut self) {
        assert_eq!(self.current(), T!['[']);
        let m = self.start(TABLE);
        self.table_header(false);
        self.entries();
        self.finish(m)
    }

    // test-array-table
    // [[array-table]]
    // a = 1
    // b = 2
    fn array_table(&mut self) {
        assert!(self.at(0) == T!['['] && self.at(1) == T!['[']);
        let m = self.start(ARRAY_TABLE);
        self.table_header(true);
        self.entries();
        self.finish(m)
    }

    // test-table-header
    // [[table . 'header']]
    fn table_header(&mut self, array: bool) {
        assert_eq!(self.current(), T!['[']);
        let m = self.start(TABLE_HEADER);
        self.bump();
        if array {
            assert_eq!(self.current(), T!['[']);
            self.bump();
        }

        self.keys();

        self.eat(R_BRACK);
        if array {
            self.eat(R_BRACK);
        }
        self.finish(m);
    }
}
