use alloc::vec::Vec;

use crate::{
    ParseTable, Transition, TransitionTable,
};

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

#[derive(Debug, Clone, Default)]
pub struct JsonFragment<'a> {
    pub fragment_type: JsonFragmentType,
    pub children: Vec<JsonFragment<'a>>,
    pub tokens: Vec<&'a Token>
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
const STATES: &'static [TransitionTable] = &[
    // Finish - State 0
    TransitionTable::new(&TransitionTable::new_all(Transition::None { next: 0 })),

    // Fail - State 1
    TransitionTable::new(&TransitionTable::new_all(Transition::None { next: 1 })),
    
    // Value - State 2
    TransitionTable::new(&[
        (LBRACE, Transition::Call { call: JsonFragmentType::Obj as usize, uncall: 3 }),  // Obj
        (LBRACK, Transition::Call { call: JsonFragmentType::Arr as usize, uncall: 3 }),  // Arr
        (STRING, Transition::Push { next: 3 }), // String
        (NUMBER, Transition::Push { next: 3 }), // Number
        (TRUE  , Transition::Push { next: 3 }), // True
        (FALSE , Transition::Push { next: 3 }), // False
        (NULL  , Transition::Push { next: 3 }), // Null
    ]),
        // State 3
        TransitionTable::new(&TransitionTable::new_all(Transition::Uncall)),


    // Obj - State 4
    TransitionTable::new(&[
        (LBRACE, Transition::None { next: 5 }),  // '{'
    ]),
        // State 5
        TransitionTable::new(&[
            (STRING, Transition::Call { call: JsonFragmentType::Pair as usize, uncall: 6 }),  // Pair
            (RBRACE, Transition::Uncall)
        ]),

        // State 6
        TransitionTable::new(
            &TransitionTable::special_case(
                (COMMA, Transition::None { next: 5 }),
                Transition::Uncall
            )
        ),

    // Pair - State 7
    TransitionTable::new(&[
        (STRING, Transition::Push { next: 8 }),
    ]),

        // State 8
        TransitionTable::new(&[
            (COLON, Transition::None { next: 9 }),
        ]),

        // State 9
        TransitionTable::new(&TransitionTable::new_all(Transition::Call { call: JsonFragmentType::Value as usize, uncall: 10 })),

        // State 10
        TransitionTable::new(&TransitionTable::new_all(Transition::Uncall)),

    // Array - State 11
    TransitionTable::new(&[
        (LBRACK, Transition::None { next: 11 }),
    ]),
        // State 12
        TransitionTable::new(&[
            (RBRACK, Transition::Uncall ),
            
            (STRING, Transition::Push { next: 13 }),
            (NUMBER, Transition::Push { next: 13 }),
            (TRUE, Transition::Push { next: 13 }),
            (FALSE, Transition::Push { next: 13 }),
            (NULL, Transition::Push { next: 13 }),
            (LBRACE, Transition::Call { call: JsonFragmentType::Obj as usize, uncall: 13 }),
            (LBRACK, Transition::Call { call: JsonFragmentType::Arr as usize, uncall: 13 }),
        ]),

        // State 13
        TransitionTable::new(
            &TransitionTable::special_case(
                (COMMA, Transition::None { next: 12 }),
                Transition::Uncall
            )
        ),
];

pub const JSON: ParseTable = ParseTable::new(2, &STATES);
