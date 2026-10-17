use crate::{
    Action::{self, Push},
    ParseTable, Transition, TransitionTable,
};

use self::Token::*;

#[repr(usize)]
#[derive(Clone, Copy)]
enum Token {
    STRING,
    NUMBER,
    TRUE,
    FALSE,
    NULL,

    COLON,

    LBRACE,
    RBRACE,

    LBRACK,
    RBRACK,
}

#[rustfmt::skip]
const STATES: &'static [TransitionTable] = &[
    // Value - State 0
    TransitionTable::new(&[
        (LBRACE as usize, Transition::new(1, Action::None)),  // Obj
        (LBRACK as usize, Transition::new(5, Action::None)),  // Arr
        (STRING as usize, Transition::new(0, Action::Reduce)), // String
        (NUMBER as usize, Transition::new(0, Action::Reduce)), // Number
        (TRUE   as usize, Transition::new(0, Action::Reduce)), // True
        (FALSE  as usize, Transition::new(0, Action::Reduce)), // False
        (NULL   as usize, Transition::new(0, Action::Reduce)), // Null
    ]),

    // Obj - State 1
    TransitionTable::new(&[
        (LBRACE as usize, Transition::new(2, Action::None)),  // '{'
    ]),
        // State 2
        TransitionTable::new(&[
            (STRING as usize, Transition::new(2, Action::Push)),  // Pair
            (RBRACE as usize, Transition::new(0, Action::Reduce))
        ]),

    // Pair - State 3
    TransitionTable::new(&[
        (STRING as usize, Transition::new(2, Action::Push)),
    ]),

        // State 4
        TransitionTable::new(&[
            (COLON as usize, Transition::new(0, Action::None)),
        ]),

    // Array - State 5
    TransitionTable::new(&[
        (LBRACK as usize, Transition::new(6, Action::Push)),
    ]),
        // State 6
        TransitionTable::new(&[
            (RBRACK as usize, Transition::new(2, Action::Reduce)),
            
            (STRING as usize, Transition::new(0, Action::None)),
            (NUMBER as usize, Transition::new(0, Action::None)),
            (TRUE as usize, Transition::new(0, Action::None)),
            (FALSE as usize, Transition::new(0, Action::None)),
            (NULL as usize, Transition::new(0, Action::Reduce)),
            (LBRACK as usize, Transition::new(5, Action::None)),
        ]),
];

pub const JSON: ParseTable = ParseTable::new(&STATES);
