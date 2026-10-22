extern crate alloc;

mod parse;
mod table;
mod json;

use std::{hint::black_box};

use crate::json::Token;
use json::JSON;

pub fn main() {
    let input = [Token::LBRACE, Token::STRING, Token::COLON, Token::NUMBER, Token::COMMA, Token::STRING, Token::COLON, Token::NUMBER,Token::RBRACE];
    
    crate::parse::kernel(&JSON, &black_box(input));
}