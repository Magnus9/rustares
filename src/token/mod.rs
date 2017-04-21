
/*
 * The token types are written in such a way that
 * allot of code can use this macro to verify whether
 * a tokentype is valid for an statement/expr.
 */
macro_rules! is_between {
    ($type:expr, $min:expr, $max:expr) => (
        $type as i32 >= $min as i32 &&
        $type as i32 <= $max as i32
    );
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq,
         PartialOrd)]
pub enum TokenType {
    // DATATYPES
    STRING,
    INTEGER,
    FLOAT,
    TRUE,
    FALSE,
    NIL,
    IDENT,

    // RESERVED WORDS
    DEF,
    IF,
    ELIF,
    ELSE,
    FOR,
    WHILE,
    UNTIL,
    IN,
    IMPORT,
    DEBUG,
    RETURN,

    // SYMBOLS
    LOGICAL_OR,
    LOGICAL_AND,
    EQL,
    NOT_EQL,
    LT,
    LE,
    GT,
    GE,
    BITWISE_OR,
    BITWISE_XOR,
    BITWISE_AND,
    LEFT_SHIFT,
    RIGHT_SHIFT,
    DOT,
    DOTDOT,
    PLUS,
    MINUS,
    MUL,
    DIV,
    MODULO,
    BANG,
    COMPL,
    LPAREN,
    RPAREN,
    LBRACK,
    RBRACK,
    LBRACE,
    RBRACE,
    COMMA,
    SEMICOLON,
    ASSIGN_ARROW,
    NEWLINE,

    // ASSIGNMENTS
    ASSIGN,
    BITWISE_OR_ASSIGN,
    BITWISE_XOR_ASSIGN,
    BITWISE_AND_ASSIGN,
    LEFT_SHIFT_ASSIGN,
    RIGHT_SHIFT_ASSIGN,
    PLUS_ASSIGN,
    MINUS_ASSIGN,
    MUL_ASSIGN,
    DIV_ASSIGN,
    MODULO_ASSIGN,

    // Imaginary tokens
    BLOCK,
    SUB_DECL,
    SUB_LITERAL,
    SUB_PARAMS,
    ARRAY_DECL,
    HASH_DECL,
    HASH_ELEM,
    CALL,
    SUBSCRIPT,
    // MINUS is changed into NEGATE on parsing time.
    NEGATE,

    EOF,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
    StringValue(String),
    IntegerValue(i64),
    FloatValue(f64),
    BoolValue(bool),
}

// A semantic bombshell :)
#[derive(Clone, PartialEq, PartialOrd)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
    pub value: Value,
    pub line_num: i32,
    pub line_pos: i32,
}

impl Token
{
    pub fn new(line_num: i32, line_pos: i32) -> Token
    {
        return Token {
            text: String::from(""),
            token_type: TokenType::STRING,
            value: Value::IntegerValue(0i64),
            line_num: line_num,
            line_pos: line_pos, 
        }
    }

    pub fn new_imag(text: String, token_type: TokenType,
                    line_num: i32, line_pos: i32)
        -> Token
    {
        /*
         * Imaginary tokens dont really require the text
         * field, but are added to give readable output
         * when generating a node string tree.
         */
        return Token {
            text: text,
            token_type: token_type,
            value: Value::IntegerValue(0i64),
            line_num: line_num,
            line_pos: line_pos,
        }
    }

    pub fn string(&self) -> String
    {
        return self.text.clone();
    }
}