use alloc::vec::Vec;
use smallvec::SmallVec;

use crate::table::{FAIL, ParseTable, TransitionTable, UNCALL};

use self::Token::*;

#[repr(usize)]
#[derive(Clone, Copy, Debug, Default)]
pub enum JsonFragmentType {
    #[default]
    Incomplete = 69,
    JsonDocument = 0,
    Value = 2,
    Obj = 4,
    Pair = 7,
    Arr = 11,
}

impl TryFrom<usize> for JsonFragmentType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let v = match value {
            69 => JsonFragmentType::Incomplete,
            0 => JsonFragmentType::JsonDocument,
            3 => JsonFragmentType::Value,
            4 => JsonFragmentType::Obj,
            6 => JsonFragmentType::Pair,
            9 => JsonFragmentType::Arr,
            _ => return Err(())
        };

        Ok(v)
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum JsonTransition {
    None { next: usize },
    Push { next: usize },
    // Uncall is for different return "addresses"
    Call { call_state: usize, return_state: usize },
    Uncall
}

#[derive(Debug, Clone, Default)]
pub struct JsonFragment<'a> {
    pub fragment_type: JsonFragmentType,
    pub children: Vec<JsonFragment<'a>>,
    pub tokens: SmallVec<[&'a Token; 8]>
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    STRING,
    NUMBER,
    TRUE,
    FALSE,
    NULL,

    COLON,
    COMMA,

    LBRACE,
    RBRACE,

    LBRACK,
    RBRACK,

    EOF
}

#[rustfmt::skip]
const STATES: &'static [TransitionTable<Token, JsonTransition>] = &[
    // Finish - State 0
    TransitionTable::new(&[], JsonTransition::None { next: 0 }),

    // Fail - State 1
    TransitionTable::new(&[], JsonTransition::None { next: 1 }),
    
    // Value - State 2
    TransitionTable::new(&[
        (LBRACE, JsonTransition::Call { call_state: JsonFragmentType::Obj as usize, return_state: 3 }),  // Obj
        (LBRACK, JsonTransition::Call { call_state: JsonFragmentType::Arr as usize, return_state: 3 }),  // Arr
        (STRING , JsonTransition::Push { next: 3 }), // String
        (NUMBER , JsonTransition::Push { next: 3 }), // Number
        (TRUE ,   JsonTransition::Push { next: 3 }), // True
        (FALSE ,  JsonTransition::Push { next: 3 }), // False
        (NULL ,   JsonTransition::Push { next: 3 }), // Null
    ], JsonTransition::None { next: FAIL }),
        // State 3
        TransitionTable::new(&[], JsonTransition::None { next: UNCALL }),


    // Obj - State 4
    TransitionTable::new(&[
        (LBRACE , JsonTransition::None { next: 5 }),  // '{'
    ], JsonTransition::None { next: FAIL }),
        // State 5
        TransitionTable::new(&[
            (STRING , JsonTransition::Call { call_state: JsonFragmentType::Pair as usize, return_state: 6 }),  // Pair
            (RBRACE , JsonTransition::Uncall)
        ], JsonTransition::None { next: FAIL }),

        // State 6
        TransitionTable::new(
            &[(COMMA , JsonTransition::None { next: 5 })],
            JsonTransition::Uncall
        ),

    // Pair - State 7
    TransitionTable::new(&[
        (STRING , JsonTransition::Push { next: 8 }),
    ], JsonTransition::None { next: FAIL }),

        // State 8
        TransitionTable::new(&[
            (COLON , JsonTransition::None { next: 9 }),
        ], JsonTransition::None { next: FAIL }),

        // State 9
        TransitionTable::new(&[], JsonTransition::Call { call_state: JsonFragmentType::Value as usize, return_state: 10 }),

        // State 10
        TransitionTable::new(&[], JsonTransition::Uncall),

    // Array - State 11
    TransitionTable::new(&[
        (LBRACK , JsonTransition::None { next: 11 }),
    ], JsonTransition::None { next: FAIL }),
        // State 12
        TransitionTable::new(&[
            (RBRACK , JsonTransition::Uncall ),
            
            (STRING , JsonTransition::Push { next: 13 }),
            (NUMBER , JsonTransition::Push { next: 13 }),
            (TRUE , JsonTransition::Push { next: 13 }),
            (FALSE , JsonTransition::Push { next: 13 }),
            (NULL , JsonTransition::Push { next: 13 }),
            (LBRACE , JsonTransition::Call { call_state: JsonFragmentType::Obj as usize, return_state: 13 }),
            (LBRACK , JsonTransition::Call { call_state: JsonFragmentType::Arr as usize, return_state: 13 }),
        ], JsonTransition::None { next: FAIL }),

        // State 13
        TransitionTable::new(
            &[(COMMA , JsonTransition::None { next: 12 })],
            JsonTransition::Uncall
        )
];

pub static JSON: ParseTable<Token, JsonTransition> = ParseTable::new(2, &STATES);
