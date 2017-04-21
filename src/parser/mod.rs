
use scanner::scanner::*;
use token::*;
use token::TokenType::*;
use intermediate::*;
use module::*;

pub struct Parser<'a> {
    scanner: &'a mut Scanner<'a>,
    module: &'a Module,
    current: Token,
    next: Token,
    in_subroutine: bool,
}

impl<'a> Parser<'a>
{
    pub fn new(scanner: &'a mut Scanner<'a>, module: &'a Module)
        -> Parser<'a>
    {
        return Parser {
            current: scanner.next_token(),
            next: scanner.next_token(),
            scanner: scanner,
            module: module,
            in_subroutine: false,
        };
    }

    fn error(&self, message: &'static str) -> !
    {
        let mut buf = String::new();

        buf.push_str(format!("{}:{}:{}: ", self.module.filename,
                             self.current.line_num,
                             self.current.line_pos).as_str());
        if self.current.token_type == NEWLINE {
            buf.push_str("unexpected newline, ");
        }
        else if self.current.token_type == EOF {
            buf.push_str("unexpected end-of-file, ");
        }
        else if is_between!(self.current.token_type,
                            STRING, IDENT) {
            buf.push_str(format!("unexpected literal near '{}', ",
                                 self.current.string()).as_str());
        }
        else if is_between!(self.current.token_type,
                            DEF, IMPORT) {
            buf.push_str(format!("unexpected keyword near '{}', ",
                                 self.current.string()).as_str());
        }
        else {
            buf.push_str(format!("unexpected symbol near '{}', ",
                                 self.current.string()).as_str());
        }
        buf.push_str(message);

        panic!(buf);
    }

    fn next_token(&mut self)
    {
        self.current = self.next.clone();
        self.next = self.scanner.next_token();
    }

    fn peek_current(&self) -> TokenType
    {
        return self.current.token_type;
    }

    fn peek_next(&self) -> TokenType
    {
        return self.next.token_type;
    }

    fn __match(&mut self, token_type: TokenType,
               message: &'static str)
    {
        if self.peek_current() != token_type {
            self.error(message);
        }
        self.next_token();
    }

    fn skip_newlines(&mut self)
    {
        while self.peek_current() == NEWLINE {
            self.next_token();
        }
    }

    fn next_and_skip_newlines(&mut self)
    {
        self.next_token();
        self.skip_newlines();
    }

    fn match_and_skip_newlines(&mut self,
                               token_type: TokenType,
                               message: &'static str)
    {
        self.__match(token_type, message);
        self.skip_newlines();
    }

    fn match_line(&mut self, message: &'static str)
    {
        self.__match(NEWLINE, message);
        self.skip_newlines();
    }

    fn is_factor(&self) -> bool
    {
        let token_type = self.peek_current();
        
        return token_type == MINUS || token_type == BANG ||
               token_type == COMPL;
    }
    
    fn statement_trailer(&mut self)
    {
        let token_type = self.peek_current();

        if token_type == SEMICOLON {
            self.next_token();
            self.skip_newlines();
        }
        else if token_type == NEWLINE {
            self.skip_newlines();
        }
        else {
            self.__match(EOF, "expected end-of-file");
        }
    }

    fn block_trailer(&mut self)
    {
        if self.peek_current() == SEMICOLON {
            self.next_token();
            self.skip_newlines();
        }
        else {
            self.match_line("expected newline");
        }
    }

    pub fn program(&mut self) -> Box<Node>
    {
        let mut program = Node::new(Token::new_imag("BLOCK".to_string(),
                                                     BLOCK,
                                                     self.current.line_num,
                                                     self.current.line_pos));
        self.skip_newlines();
        while self.peek_current() != EOF {
            if self.peek_current() == DEF && self.peek_next() != LPAREN {
                program.add_child(self.def_statement(false));
            }
            else {
                program.add_child(self.statement());
            }
            self.statement_trailer();
        }
        return program;
    }

    fn statement(&mut self) -> Box<Node>
    {
        return match self.peek_current() {
            IF => self.if_statement(),
            WHILE | UNTIL => self.control_statement(),
            FOR    => self.for_statement(),
            IMPORT => self.import_statement(),
            DEBUG  => self.debug_statement(),
            RETURN => self.return_statement(),
            _      => self.expr_statement(),
        }
    }

    /*
     * The parameter is_literal is passed for code-reusage,
     * the only difference between a named subroutine and a
     * literal is the identifier which is scanned/not scanned
     * based on the value passed.
     */
    fn def_statement(&mut self, is_literal: bool) -> Box<Node>
    {
        let mut node: Box<Node>;
        self.next_token();

        if !is_literal {
            node = Node::new(Token::new_imag("SUB_DECL".to_string(),
                                              SUB_DECL,
                                              self.current.line_num,
                                              self.current.line_pos));
            if self.peek_current() != IDENT {
                self.error("expected identifier");
            }
            node.add_child(Node::new(self.current.clone()));
            self.next_token();
        }
        else {
            node = Node::new(Token::new_imag("SUB_LITERAL".to_string(),
                                              SUB_LITERAL,
                                              self.current.line_num,
                                              self.current.line_pos));
        }
        self.match_and_skip_newlines(LPAREN,
                                     "expected '(' to open parameter list");

        let mut params = Node::new(Token::new_imag("SUB_PARAMS".to_string(),
                                                    SUB_PARAMS,
                                                    self.current.line_num,
                                                    self.current.line_pos));
        for n in self.parameter_list() {
            params.add_child(n);
        }
        self.skip_newlines();
        self.__match(RPAREN, "expected ')' to close parameter list");
        
        node.add_child(params);

        self.in_subroutine = true;
        node.add_child(self.block());
        self.in_subroutine = false;

        return node;
    }

    fn parameter_list(&mut self) -> Vec<Box<Node>>
    {
        let mut sequence: Vec<Box<Node>> = Vec::new();

        if self.peek_current() == RPAREN {
            return sequence;
        }
        loop {
            if self.peek_current() != IDENT {
                self.error("expected identifier as argument");
            }
            sequence.push(Node::new(self.current.clone()));
            self.next_token();
            if self.peek_current() != COMMA {
                break;
            }
            self.next_and_skip_newlines();
        }
        return sequence;
    }

    fn if_statement(&mut self) -> Box<Node>
    {
        let mut node = Node::new(self.current.clone());
        self.next_token();

        node.add_child(self.expr());
        node.add_child(self.block());

        let mut elif_root = Node::new(Token::new_imag("ELIF".to_string(),
                                                       ELIF,
                                                       self.current.line_num,
                                                       self.current.line_pos));
        while self.peek_current() == ELIF {
            self.next_token();

            elif_root.add_child(self.expr());
            elif_root.add_child(self.block());
        }
        node.add_child(elif_root);
        if self.peek_current() == ELSE {
            self.next_token();
            node.add_child(self.block());
        }
        return node;
    }

    fn control_statement(&mut self) -> Box<Node>
    {
        let mut node = Node::new(self.current.clone());
        self.next_token();

        node.add_child(self.expr());
        node.add_child(self.block());

        return node;
    }

    fn for_statement(&mut self) -> Box<Node>
    {
        let mut node = Node::new(self.current.clone());
        self.next_token();

        if self.peek_current() != IDENT {
            self.error("expected identifier");
        }
        node.add_child(Node::new(self.current.clone()));
        self.next_token();

        self.__match(IN, "expected keyword 'in' before expression");
        node.add_child(self.expr());
        node.add_child(self.block());

        return node;
    }

    fn import_statement(&mut self) -> Box<Node>
    {
        let mut node = Node::new(self.current.clone());
        self.next_token();
        node.add_child(self.expr());

        return node;
    }

    fn debug_statement(&mut self) -> Box<Node>
    {
        /*
         * The debug statement is just a statement that
         * outputs information about an ares object. This
         * will help to verify values during development. This
         * will be replaced by some builtin subroutines in later
         * stages.
         */
        let mut node = Node::new(self.current.clone());
        self.next_token();
        node.add_child(self.expr());

        return node;
    }

    fn return_statement(&mut self) -> Box<Node>
    {
        if !self.in_subroutine {
            self.error("'return' outside subroutine");
        }
        let mut node = Node::new(self.current.clone());
        self.next_token();

        let token_type = self.peek_current();

        if token_type != NEWLINE && token_type != SEMICOLON &&
           token_type != EOF {
            node.add_child(self.expr());
        }
        return node;
    }

    fn expr_statement(&mut self) -> Box<Node>
    {
        let node = self.expr();

        return node;
    }

    fn block(&mut self) -> Box<Node>
    {
        self.skip_newlines();
        self.__match(LBRACE, "expected '{' to open block");

        let mut node = Node::new(Token::new_imag("BLOCK".to_string(),
                                                  BLOCK,
                                                  self.current.line_num,
                                                  self.current.line_pos));
        self.skip_newlines();
        while self.peek_current() != RBRACE &&
              self.peek_current() != EOF {
            node.add_child(self.statement());

            self.block_trailer();
        }
        self.__match(RBRACE, "expected '}' to close block");

        return node;
    }

    fn expr(&mut self) -> Box<Node>
    {
        return self.assignment_expr();
    }
    
    fn assignment_expr(&mut self) -> Box<Node>
    {
        let mut left = self.range_expr();
        if self.peek_current() == ASSIGN {
            match left.get_type() {
                SUBSCRIPT | IDENT => (),
                _ => self.error(""),
            }
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.range_expr());
        }
        return left;
    }

    fn range_expr(&mut self) -> Box<Node>
    {
        let mut left = self.or_expr();
        while self.peek_current() == DOTDOT {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.or_expr());
        }
        return left;
    }

    fn or_expr(&mut self) -> Box<Node>
    {
        let mut left = self.and_expr();
        while self.peek_current() == LOGICAL_OR {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.and_expr());
        }
        return left;
    }

    fn and_expr(&mut self) -> Box<Node>
    {
        let mut left = self.eql_expr();
        while self.peek_current() == LOGICAL_AND {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.eql_expr());
        }
        return left;
    }

    fn eql_expr(&mut self) -> Box<Node>
    {
        let mut left = self.comp_expr();
        while self.peek_current() == EQL ||
              self.peek_current() == NOT_EQL {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.comp_expr());
        }
        return left;
    }

    fn comp_expr(&mut self) -> Box<Node>
    {
        let mut left = self.bit_or_expr();
        while is_between!(self.peek_current(), LT, GE) {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.bit_or_expr());
        }
        return left;
    }

    fn bit_or_expr(&mut self) -> Box<Node>
    {
        let mut left = self.xor_expr();
        while self.peek_current() == BITWISE_OR {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.xor_expr());
        }
        return left;
    }

    fn xor_expr(&mut self) -> Box<Node>
    {
        let mut left = self.bit_and_expr();
        while self.peek_current() == BITWISE_XOR {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.bit_and_expr());
        }
        return left;
    }

    fn bit_and_expr(&mut self) -> Box<Node>
    {
        let mut left = self.shift_expr();
        while self.peek_current() == BITWISE_AND {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.shift_expr());
        }
        return left;
    }

    fn shift_expr(&mut self) -> Box<Node>
    {
        let mut left = self.arith_expr();
        while self.peek_current() == LEFT_SHIFT ||
              self.peek_current() == RIGHT_SHIFT {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.arith_expr());
        }
        return left;
    }

    fn arith_expr(&mut self) -> Box<Node>
    {
        let mut left = self.term_expr();
        while self.peek_current() == PLUS ||
              self.peek_current() == MINUS {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.term_expr());
        }
        return left;
    }

    fn term_expr(&mut self) -> Box<Node>
    {
        let mut left = self.factor_expr();
        while is_between!(self.peek_current(), MUL,
                          MODULO) {
            let op_node = Node::new(self.current.clone());
            left = left.get_root(op_node);

            self.next_and_skip_newlines();
            left.add_child(self.factor_expr());
        }
        return left;
    }

    fn factor_expr(&mut self) -> Box<Node>
    {
        if self.is_factor() {
            /*
             * is_factor uses a '-' (minus) tokentype to verify
             * if it is a factor unit, amongst other types. If this
             * is the case, change the type into imaginary
             * TokenType::NEGATE.
             */
            if self.peek_current() == MINUS {
                self.current.token_type = NEGATE;
            }
            let mut left = Node::new(self.current.clone());
            self.next_and_skip_newlines();
            if self.is_factor() {
                // Recurse factor units
                left.add_child(self.factor_expr());
            }
            else {
                left.add_child(self.trailer_expr());
            }
            return left;
        }
        return self.trailer_expr();
    }

    fn trailer_expr(&mut self) -> Box<Node>
    {
        let mut left = self.atom();
        loop {
            if self.peek_current() == LBRACK {
                left = self.subscript(left);
            }
            else if self.peek_current() == LPAREN {
                left = self.call_literal(left);
            }
            else {
                break;
            }
        }
        return left;
    }
    
    fn atom(&mut self) -> Box<Node>
    {
        /*
         * Since we are using homogenous nodes which uses
         * a token for semantic information in later stages +
         * our token values was defined in the lexical stage,
         * there is not much to do when casing primitive types
         * other than just defining a new node with their underlying
         * token.
         */
        let node: Box<Node>;

        match self.peek_current() {
            STRING | INTEGER | FLOAT | TRUE | FALSE | NIL |
            IDENT  => {
                node = Node::new(self.current.clone());
                self.next_token();
            },
            LBRACK => node = self.array_literal(),
            LBRACE => node = self.hash_literal(),
            LPAREN => node = self.grouping(),
            DEF    => node = self.def_statement(true),
            _      => self.error("expected expression"),
        }
        return node;
    }
    
    fn grouping(&mut self) -> Box<Node>
    {
        self.next_token();
        let node = self.expr();
        self.__match(RPAREN, "expected ')'");

        return node;
    }

    fn subscript(&mut self, left: Box<Node>) -> Box<Node>
    {
        let mut node = Node::new(Token::new_imag("SUBSCRIPT".to_string(),
                                                  SUBSCRIPT,
                                                  self.current.line_num,
                                                  self.current.line_pos));
        node = left.get_root(node);
        
        self.next_and_skip_newlines();
        node.add_child(self.expr());
        self.skip_newlines();

        self.__match(RBRACK, "expected ']' to close subscript");

        return node;
    }

    fn call_literal(&mut self, left: Box<Node>) -> Box<Node>
    {
        let mut node = Node::new(Token::new_imag("CALL".to_string(),
                                                  CALL,
                                                  self.current.line_num,
                                                  self.current.line_pos));
        node = left.get_root(node);
        self.next_and_skip_newlines();

        for n in self.expression_list(RPAREN) {
            node.add_child(n);
        }
        self.skip_newlines();
        self.__match(RPAREN, "expected ')' to close the function call");

        return node;
    }

    fn array_literal(&mut self) -> Box<Node>
    {
        let mut node = Node::new(Token::new_imag("ARRAY_DECL".to_string(),
                                                  ARRAY_DECL,
                                                  self.current.line_num,
                                                  self.current.line_pos));
        self.next_and_skip_newlines();
        for n in self.expression_list(RBRACK) {
            node.add_child(n);
        }
        self.skip_newlines();
        self.__match(RBRACK, "expected ']' to close array literal");

        return node;
    }

    fn hash_literal(&mut self) -> Box<Node>
    {
        let mut node = Node::new(Token::new_imag("HASH_DECL".to_string(),
                                                  HASH_DECL,
                                                  self.current.line_num,
                                                  self.current.line_pos));
        self.next_and_skip_newlines();
        if self.peek_current() == RBRACE {
            self.next_token();

            return node;
        }
        loop {
            let mut elem = Node::new(Token::new_imag("HASH_ELEM".to_string(),
                                                      HASH_ELEM,
                                                      self.current.line_num,
                                                      self.current.line_pos));
            elem.add_child(self.expr());
            self.__match(ASSIGN_ARROW, "expected '=>'");
            elem.add_child(self.expr());

            node.add_child(elem);
            if self.peek_current() != COMMA {
                break;
            }
            self.next_and_skip_newlines();
        }
        self.skip_newlines();
        self.__match(RBRACE, "expected '}' to close hash literal");

        return node;
    }

    fn expression_list(&mut self, end: TokenType) -> Vec<Box<Node>>
    {
        let mut sequence: Vec<Box<Node>> = Vec::new();

        if self.peek_current() == end {
            return sequence;
        }
        loop {
            sequence.push(self.expr());
            if self.peek_current() != COMMA {
                break;
            }
            self.next_and_skip_newlines();
        }
        return sequence;
    }
}