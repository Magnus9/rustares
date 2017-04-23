
use std::collections::HashMap;
use std::str::FromStr;
use token::*;
use token::TokenType::*;
use token::Value::*;
use module::Module;

const EOF_CHAR: char = '\0';

macro_rules! get_literal {
    ($program:expr, $s:expr, $e:expr) => (
        $program[$s as usize..$e as usize].to_string();
    );
}

macro_rules! create_map {
    ($($key:expr => $value:expr),+) => (
        {
            let mut m = HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    );
}

pub struct Scanner<'a> {
    program: &'a str,
    input: &'a [u8],
    module: &'a Module,
    line_num: i32,
    line_pos: i32,
    position: i32,
    ch: char,
    reserved_words: HashMap<&'static str,
                            TokenType>,
}

impl<'a> Scanner<'a>
{
    pub fn new(program: &'a str, module: &'a Module)
        -> Scanner<'a>
    {
        let mut scanner = Scanner {
            program: program,
            input: program.as_bytes(),
            module: module,
            line_num: 1,
            line_pos: 0,
            position: -1,
            ch: '\0',
            reserved_words: create_map!(
                "def"    => DEF,
                "if"     => IF,
                "elif"   => ELIF,
                "else"   => ELSE,
                "for"    => FOR,
                "while"  => WHILE,
                "until"  => UNTIL,
                "switch" => SWITCH,
                "case"   => CASE,
                "default"=> DEFAULT,
                "in"     => IN,
                "import" => IMPORT,
                "true"   => TRUE,
                "false"  => FALSE,
                "nil"    => NIL,
                "debug"  => DEBUG,
                "return" => RETURN
            ),
        };
        scanner.next_char();

        return scanner;
    }

    fn error(&self, line_num: i32, line_pos: i32,
             message: String)
    {
        /*
         * For now we panic! once we locate a scanner
         * error. Later we will patch in inline assembly
         * jumping, to get out of heavy recursion.
         */
        panic!("{}:{}:{}: {}", self.module.filename, line_num,
               line_pos, message.as_str());
    }

    fn get_char(&self, position: usize) -> char
    {
        return self.input[position] as char;
    }

    fn next_char(&mut self) -> char
    {
        self.position += 1;
        if self.position == self.program.len() as i32 {
            self.ch = EOF_CHAR;
        } else {
            self.ch = self.get_char(self.position as usize);
            if self.ch == '\n' {
                self.line_num += 1;
            }
            self.line_pos += 1;
        }
        return self.ch;
    }

    fn peek_char(&self, num: i32) -> char
    {
        let new_pos = self.position + num;
        if new_pos >= self.program.len() as i32 {
            return EOF_CHAR;
        }
        return self.get_char(new_pos as usize);
    }

    fn next_charx(&mut self, num: i32)
    {
        let mut i = 0;

        while i < num {
            self.next_char();
            i += 1
        }
    }

    /*
     * A whitespace is equal to a space, \t, or \r. If
     * it finds '#' it loops until '\n' or '\0'.
     */
    fn whitespace(&mut self)
    {
        while self.ch == ' '  || self.ch == '\r' ||
              self.ch == '\t' || self.ch == '#' {
            if self.ch == '#' {
                while self.ch != '\n' && self.ch != EOF_CHAR {
                    self.next_char();
                }
            } else {
                self.next_char();
            }
        }
    }

    /*
     * Method parses a long comment '==='.
     */
    fn long_comment(&mut self)
    {
        self.next_charx(3);
        while self.ch != EOF_CHAR {
            if self.ch == '=' {
                if self.peek_char(1) == '=' && self.peek_char(2) == '=' {
                    break;
                }
            }
            self.next_char();
        }
        if self.ch == EOF_CHAR {
            self.error(self.line_num, self.line_pos,
                       "unterminated long comment".to_string());
        }
        self.next_charx(3);
    }

    /*
     * next_token returns a token filled with semantic
     * information. It starts by skipping whitespace /
     * comments and declares the token. The token will
     * be filled with data through the routine.
     */
    pub fn next_token(&mut self) -> Token
    {
        self.whitespace();
        if self.is_long_comment() {
            self.long_comment();
        }
        let mut token = Token::new(self.line_num, self.line_pos);

        if self.ch == EOF_CHAR {
            token.text = "".to_string();
            token.token_type = EOF;
        }
        else if self.is_letter() {
            self.word_token(&mut token);
        }
        else if self.is_hex() {
            self.number_token_hex(&mut token);
        }
        else if self.is_digit() {
            self.number_token(&mut token);
        }
        else if self.ch == '"' || self.ch == '\'' {
            self.string_token(&mut token);
        }
        else {
            token.text.push(self.ch);
            match self.ch {
                '|' => {
                    if self.peek_char(1) == '|' {
                        token.text.push(self.next_char());
                        token.token_type = LOGICAL_OR;
                    }
                    else if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = BITWISE_OR_ASSIGN;
                    }
                    else {
                        token.token_type = BITWISE_OR;
                    }
                },
                '&' => {
                    if self.peek_char(1) == '&' {
                        token.text.push(self.next_char());
                        token.token_type = LOGICAL_AND;
                    }
                    else if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = BITWISE_AND_ASSIGN;
                    }
                    else {
                        token.token_type = BITWISE_AND;
                    }
                },
                '=' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = EQL;
                    }
                    else if self.peek_char(1) == '>' {
                        token.text.push(self.next_char());
                        token.token_type = ASSIGN_ARROW;
                    }
                    else {
                        token.token_type = ASSIGN;
                    }
                },
                '!' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = NOT_EQL;
                    }
                    else {
                        token.token_type = BANG;
                    }
                },
                '<' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = LE;
                    }
                    else if self.peek_char(1) == '<' {
                        token.text.push(self.next_char());
                        if self.peek_char(1) == '=' {
                            token.text.push(self.next_char());
                            token.token_type = LEFT_SHIFT_ASSIGN;
                        }
                        else {
                            token.token_type = LEFT_SHIFT;
                        }
                    }
                    else {
                        token.token_type = LT;
                    }
                },
                '>' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = GE;
                    }
                    else if self.peek_char(1) == '>' {
                        token.text.push(self.next_char());
                        if self.peek_char(1) == '=' {
                            token.text.push(self.next_char());
                            token.token_type = RIGHT_SHIFT_ASSIGN;
                        }
                        else {
                            token.token_type = RIGHT_SHIFT;
                        }
                    }
                    else {
                        token.token_type = GT;
                    }
                },
                '^' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = BITWISE_XOR_ASSIGN;
                    }
                    else {
                        token.token_type = BITWISE_XOR;
                    }
                },
                '.' => {
                    if self.peek_char(1) == '.' {
                        token.text.push(self.next_char());
                        token.token_type = DOTDOT;
                    }
                    else {
                        token.token_type = DOT;
                    }
                },
                '+' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = PLUS_ASSIGN;
                    }
                    else {
                        token.token_type = PLUS;
                    }
                },
                '-' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = MINUS_ASSIGN;
                    }
                    else {
                        token.token_type = MINUS;
                    }
                },
                '*' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = MUL_ASSIGN;
                    }
                    else {
                        token.token_type = MUL;
                    }
                },
                '/' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = DIV_ASSIGN;
                    }
                    else {
                        token.token_type = DIV;
                    }
                },
                '%' => {
                    if self.peek_char(1) == '=' {
                        token.text.push(self.next_char());
                        token.token_type = MODULO_ASSIGN;
                    }
                    else {
                        token.token_type = MODULO;
                    }
                },
                '~'  => token.token_type = COMPL,
                '('  => token.token_type = LPAREN,
                ')'  => token.token_type = RPAREN,
                '['  => token.token_type = LBRACK,
                ']'  => token.token_type = RBRACK,
                '{'  => token.token_type = LBRACE,
                '}'  => token.token_type = RBRACE,
                ','  => token.token_type = COMMA,
                ';'  => token.token_type = SEMICOLON,
                '\n' => {
                    token.token_type = NEWLINE;
                    token.line_num -= 1; self.line_pos = 0;
                },
                _    => self.error(self.line_num, self.line_pos,
                                   format!("unrecognized character '{}'",
                                   self.ch)),
            }
            self.next_char();
        }
        return token;
    }

    pub fn word_token(&mut self, token: &mut Token)
    {
        let position = self.position;

        while self.is_letter() {
            self.next_char();
        }
        token.text = get_literal!(self.program, position,
                                  self.position);
        if let Some(word) = self.reserved_words.get(
            token.text.as_str())
        {
            token.token_type = *word;
            match word {
                &TRUE  => token.value = BoolValue(true),
                &FALSE => token.value = BoolValue(false),
                _ => (),
            }
        }
        else {
            token.token_type = IDENT;
        }
    }

    pub fn number_token(&mut self, token: &mut Token)
    {
        token.token_type = INTEGER;

        let position = self.position;
        while self.is_digit() {
            self.next_char();
        }
        if self.ch == '.' && self.peek_char(1) != '.' {
            self.next_char();
            while self.is_digit() {
                self.next_char();
            }
            token.token_type = FLOAT;
        }
        token.text = get_literal!(self.program, position,
                                  self.position);
        if token.token_type == INTEGER {
            let value = i64::from_str_radix(token.text.as_str(), 10);
            if value.is_err() {
                self.error(token.line_num, token.line_pos,
                           format!("number literal was too large"));
            }
            token.value = IntegerValue(value.unwrap());
        }
        else {
            let value = f64::from_str(token.text.as_str());
            token.value = FloatValue(value.unwrap());
        }
    }

    fn number_token_hex(&mut self, token: &mut Token)
    {
        let position = self.position;

        self.next_charx(2);
        while self.read_hexdigit() != 1 {
            self.next_char();
        }
        token.text = get_literal!(self.program, position,
                                  self.position);
        token.token_type = INTEGER;

        let value = i64::from_str_radix(&token.text[2..], 16);
        if value.is_err() {
            self.error(token.line_num, token.line_pos,
                       format!("number literal was too large"));
        }
        token.value = IntegerValue(value.unwrap());
    }

    pub fn read_hex_escape(&mut self, delimit: char) -> char
    {
        let mut value = 0;

        for _ in 0..2 {
            self.next_char();
            if self.ch == delimit || self.ch == EOF_CHAR {
                self.error(self.line_num, self.line_pos,
                           "incomplete hex escape sequence".to_string());
            }
            let digit = self.read_hexdigit();

            if digit == -1 {
                self.error(self.line_num, self.line_pos,
                           "incomplete hex escape sequence".to_string());
            }
            value = (value * 16) + digit;
        }
        return value as u8 as char;
    }

    pub fn string_token(&mut self, token: &mut Token)
    {
        let mut buf = String::new();
        let delimit = self.ch;

        self.next_char();
        while self.ch != delimit && self.ch != EOF_CHAR {
            if self.ch == '\\' {
                self.next_char();
                match self.ch {
                    '"'  => buf.push('"'),
                    '\\' => buf.push('\\'),
                    '\'' => buf.push('\''),
                    'n'  => buf.push('\n'),
                    'r'  => buf.push('\r'),
                    't'  => buf.push('\t'),
                    'x'  => buf.push(self.read_hex_escape(delimit)),
                    _    => self.error(self.line_num, self.line_pos,
                                       format!("invalid escape character {}",
                                               self.ch)),
                };
            }
            else {
                buf.push(self.ch);
            }
            self.next_char();
        }
        if self.ch == EOF_CHAR {
            self.error(self.line_num, self.line_pos,
                       "unterminated string literal".to_string());
        }
        self.next_char();
        token.text = buf;
        token.token_type = STRING;
        token.value = StringValue(token.text.clone());
    }

    fn is_letter(&self) -> bool
    {
        return self.ch >= 'a' && self.ch <= 'z' ||
               self.ch >= 'A' && self.ch <= 'Z' || self.ch == '_';
    }

    fn is_digit(&self) -> bool
    {
        return self.ch >= '0' && self.ch <= '9';
    }

    fn is_hex(&self) -> bool
    {
        let next_char = self.peek_char(1);

        return self.ch == '0' && next_char == 'x' ||
               next_char == 'X';
    }

    fn read_hexdigit(&self) -> i32
    {
        if self.ch >= '0' && self.ch <= '9' {
            return self.ch as i32 - '0' as i32;
        }
        if self.ch >= 'a' && self.ch <= 'f' {
            return self.ch as i32 - 'a' as i32 + 10;
        }
        if self.ch >= 'A' && self.ch <= 'F' {
            return self.ch as i32 - 'A' as i32 + 10;
        }
        return -1;
    }

    fn is_long_comment(&self) -> bool
    {
        return self.ch == '=' && self.peek_char(1) == '=' &&
               self.peek_char(2) == '=';
    }
}