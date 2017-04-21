
/*
 * Test that the scanner provides the correct tokens
 * in an easy way, ie, that it is production ready.
 */
use scanner::scanner::*;
use token::*;
use token::TokenType::*;
use module::Module;

macro_rules! create_tests {
    ($($text:expr, $token_type:expr),+) => (
        {
            let tests = [
            $(
                TokenMatcher::new($text, $token_type),
            )+
            ];
            tests
        }
    );
}

pub struct TokenMatcher {
    expected_text: &'static str,
    expected_type: TokenType,
}

impl TokenMatcher
{
    fn new(text: &'static str, expected_type: TokenType)
        -> TokenMatcher
    {
        return TokenMatcher {
            expected_text: text,
            expected_type: expected_type,
        };
    }

    pub fn match_reserved_words()
    {
        let tests = create_tests!("\n", NEWLINE,
                                  "if", IF,
                                  "elif", ELIF,
                                  "else", ELSE,
                                  "while", WHILE,
                                  "until", UNTIL,
                                  "in", IN,
                                  "for", FOR,
                                  "import", IMPORT);
        println!("Starting match_reserved_words() test..");
        TokenMatcher::__match(&tests, "
                              if elif else while until in for \
                              import");
        println!("Ending match_reserved_words() test..");
    }

    pub fn match_datatypes()
    {
        let tests = create_tests!("100", INTEGER,
                                  "200.452", FLOAT,
                                  "1.", FLOAT,
                                  "randomid", IDENT,
                                  "Hello", STRING,
                                  "world", STRING,
                                  "\n", NEWLINE,
                                  "true", TRUE,
                                  "false", FALSE,
                                  "nil", NIL,
                                  "0x4129", INTEGER,
                                  "\n", NEWLINE,
                                  "", EOF);
        println!("Starting match_datatypes() test..");
        TokenMatcher::__match(&tests, "100 200.452 1. randomid \"Hello\" \
                              'world'
                              true false nil 0x4129
                              ");
        println!("Ending match_datatypes() test..");
    }

    pub fn match_symbols()
    {
        let tests = create_tests!("\n", NEWLINE,
                                  "+", PLUS,
                                  "-", MINUS,
                                  "*", MUL,
                                  ">>=", RIGHT_SHIFT_ASSIGN,
                                  "<<=", LEFT_SHIFT_ASSIGN,
                                  "/=", DIV_ASSIGN,
                                  "%", MODULO,
                                  "%=", MODULO_ASSIGN,
                                  "[", LBRACK,
                                  "", EOF);
        println!("Starting match_symbols() test..");
        TokenMatcher::__match(&tests, "
                              + - * >>= <<= /= % %= [");
        println!("Ending match_symbols() test..");
    }

    pub fn match_all()
    {
        TokenMatcher::match_reserved_words();
        TokenMatcher::match_datatypes();
        TokenMatcher::match_symbols();
    }

    fn __match(tests: &[TokenMatcher], input: &'static str)
    {
        let module = Module::new("tokenmatcher".to_string());
        let mut scanner = Scanner::new(input, &module);

        let mut i = 0;
        for tt in tests {
            let token = scanner.next_token();

            if token.text != tt.expected_text {
                println!("{}. text({}) != expected text({})",
                         i, token.text, tt.expected_text);
            }
            if token.token_type != tt.expected_type {
                println!("{}. type({:?}) != expected type({:?})",
                         i, token.token_type, tt.expected_type);
            }
            i += 1
        }
    }
}