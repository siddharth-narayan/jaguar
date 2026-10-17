use crate::{
    ParseTable, Transition, TransitionTable,
};

use self::Token::*;

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
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
}
#[repr(usize)]
#[derive(Clone, Copy)]
enum Rule {
    Value = 0,
    Obj = 4,
    Pair = 6,
    Arr = 8,
}

#[rustfmt::skip]
const STATES: &'static [TransitionTable] = &[
    // Value - State 0
    TransitionTable::new(&[
        (LBRACE as usize, Transition::Call { call: Rule::Obj as usize }),  // Obj
        (LBRACK as usize, Transition::Call { call: Rule::Arr as usize }),  // Arr
        (STRING as usize, Transition::Push { next: ParseTable::UNCALL }), // String
        (NUMBER as usize, Transition::Push { next: ParseTable::UNCALL }), // Number
        (TRUE   as usize, Transition::Push { next: ParseTable::UNCALL }), // True
        (FALSE  as usize, Transition::Push { next: ParseTable::UNCALL }), // False
        (NULL   as usize, Transition::Push { next: ParseTable::UNCALL }), // Null
    ]),

    // Finish - State 1
    TransitionTable::new(&[
        (LBRACE as usize, Transition::None { next: 1 }),
        (LBRACK as usize, Transition::None { next: 1 }),
        (STRING as usize, Transition::None { next: 1 }),
        (NUMBER as usize, Transition::None { next: 1 }),
        (TRUE   as usize, Transition::None { next: 1 }),
        (FALSE  as usize, Transition::None { next: 1 }),
        (NULL   as usize, Transition::None { next: 1 }),
    ]),

    // Fail - State 2
    TransitionTable::new(&[
        (LBRACE as usize, Transition::None { next: 2 }),
        (LBRACK as usize, Transition::None { next: 2 }),
        (STRING as usize, Transition::None { next: 2 }),
        (NUMBER as usize, Transition::None { next: 2 }),
        (TRUE   as usize, Transition::None { next: 2 }),
        (FALSE  as usize, Transition::None { next: 2 }),
        (NULL   as usize, Transition::None { next: 2 }),
    ]),

    // Uncall - State 3
    TransitionTable::new(&[
        (LBRACE as usize, Transition::Uncall),
        (LBRACK as usize, Transition::Uncall),
        (STRING as usize, Transition::Uncall),
        (NUMBER as usize, Transition::Uncall),
        (TRUE   as usize, Transition::Uncall),
        (FALSE  as usize, Transition::Uncall),
        (NULL   as usize, Transition::Uncall),
    ]),

    // Obj - State 4
    TransitionTable::new(&[
        (LBRACE as usize, Transition::None { next: 5 }),  // '{'
    ]),
        // State 5
        TransitionTable::new(&[
            (STRING as usize, Transition::Push { next: 6 }),  // Pair
            (RBRACE as usize, Transition::Uncall)
        ]),

    // Pair - State 6
    TransitionTable::new(&[
        (STRING as usize, Transition::Push { next: 7 }),
    ]),

        // State 7
        TransitionTable::new(&[
            (COLON as usize, Transition::Call { call: 0 }),
        ]),

    // Array - State 8
    TransitionTable::new(&[
        (LBRACK as usize, Transition::None { next: 9 }),
    ]),
        // State 9
        TransitionTable::new(&[
            (RBRACK as usize, Transition::Uncall ),
            
            (STRING as usize, Transition::Push { next: 10 }),
            (NUMBER as usize, Transition::Push { next: 10 }),
            (TRUE as usize, Transition::Push { next: 10 }),
            (FALSE as usize, Transition::Push { next: 10 }),
            (NULL as usize, Transition::Push { next: 10 }),
            (LBRACK as usize, Transition::Call { call: 8 }), // Does this miss commas?
        ]),

        // State 10
        TransitionTable::new(&[
            (COMMA as usize, Transition::None { next: 9 } ),
        ]),
];

pub const JSON: ParseTable = ParseTable::new(&STATES);
