
/*
 * Ares uses Homogenous nodes instead of Heterogenous,
 * so some heavy constructs might use a symbol table
 * if they are tendersome to interpret instead of a
 * subtree.
 */
use token::*;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Node {
    pub token: Token,
    pub children: Vec<Box<Node>>,
}

impl Node
{
    pub fn new(token: Token) -> Box<Node>
    {
        let node = Node {
            token: token,
            children: Vec::new(),
        };
        return Box::new(node);
    }

    pub fn add_child(&mut self, node: Box<Node>)
    {
        self.children.push(node);
    }

    pub fn get_root(self, mut node: Box<Node>) -> Box<Node>
    {
        node.add_child(Box::new(self));

        return node;
    }

    pub fn string(&self) -> String
    {
        return self.token.text.clone();
    }

    pub fn get_type(&self) -> TokenType
    {
        return self.token.token_type;
    }

    pub fn get_value(&self) -> Value
    {
        return self.token.value.clone();
    }

    pub fn to_string_tree(&mut self) -> String
    {
        if self.children.len() != 0 {
            let mut buf = String::new();
            buf.push('(');
            buf.push_str((self.string() + " ").as_str());
            
            let mut i = 0;
            while i < self.children.len() {
                if i > 0 {
                    buf.push(' ');
                }
                buf.push_str(self.children[i].to_string_tree().as_str());
                i += 1;
            }
            buf.push(')');
            return buf;
        }
        return self.string();
    }
}