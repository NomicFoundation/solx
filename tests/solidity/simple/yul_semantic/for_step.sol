//! { "cases": [ {
//!     "name": "step_empty",
//!     "inputs": [
//!         {
//!             "method": "step_empty",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_assignment",
//!     "inputs": [
//!         {
//!             "method": "step_assignment",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "10"
//!     ]
//! }, {
//!     "name": "step_multi_assignment",
//!     "inputs": [
//!         {
//!             "method": "step_multi_assignment",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "10"
//!     ]
//! }, {
//!     "name": "step_builtin_call",
//!     "inputs": [
//!         {
//!             "method": "step_builtin_call",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "15"
//!     ]
//! }, {
//!     "name": "step_function_call",
//!     "inputs": [
//!         {
//!             "method": "step_function_call",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_let",
//!     "inputs": [
//!         {
//!             "method": "step_let",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "3"
//!     ]
//! }, {
//!     "name": "step_let_uninit",
//!     "inputs": [
//!         {
//!             "method": "step_let_uninit",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_let_multi",
//!     "inputs": [
//!         {
//!             "method": "step_let_multi",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "2"
//!     ]
//! }, {
//!     "name": "step_if",
//!     "inputs": [
//!         {
//!             "method": "step_if",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "4"
//!     ]
//! }, {
//!     "name": "step_if_leave",
//!     "inputs": [
//!         {
//!             "method": "step_if_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "3"
//!     ]
//! }, {
//!     "name": "step_switch_default",
//!     "inputs": [
//!         {
//!             "method": "step_switch_default",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "4"
//!     ]
//! }, {
//!     "name": "step_switch_no_default",
//!     "inputs": [
//!         {
//!             "method": "step_switch_no_default",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "4"
//!     ]
//! }, {
//!     "name": "step_switch_only_default",
//!     "inputs": [
//!         {
//!             "method": "step_switch_only_default",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_switch_case_leave",
//!     "inputs": [
//!         {
//!             "method": "step_switch_case_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "102"
//!     ]
//! }, {
//!     "name": "step_switch_default_leave",
//!     "inputs": [
//!         {
//!             "method": "step_switch_default_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "102"
//!     ]
//! }, {
//!     "name": "step_nested_for",
//!     "inputs": [
//!         {
//!             "method": "step_nested_for",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "2"
//!     ]
//! }, {
//!     "name": "step_nested_for_break_continue",
//!     "inputs": [
//!         {
//!             "method": "step_nested_for_break_continue",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "3"
//!     ]
//! }, {
//!     "name": "step_nested_for_step_leave",
//!     "inputs": [
//!         {
//!             "method": "step_nested_for_step_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "101"
//!     ]
//! }, {
//!     "name": "step_nested_for_body_leave",
//!     "inputs": [
//!         {
//!             "method": "step_nested_for_body_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "102"
//!     ]
//! }, {
//!     "name": "step_nested_for_then_leave",
//!     "inputs": [
//!         {
//!             "method": "step_nested_for_then_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "121"
//!     ]
//! }, {
//!     "name": "step_block",
//!     "inputs": [
//!         {
//!             "method": "step_block",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_block_leave",
//!     "inputs": [
//!         {
//!             "method": "step_block_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "102"
//!     ]
//! }, {
//!     "name": "step_leave",
//!     "inputs": [
//!         {
//!             "method": "step_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "101"
//!     ]
//! }, {
//!     "name": "step_leave_unreachable_tail",
//!     "inputs": [
//!         {
//!             "method": "step_leave_unreachable_tail",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "step_leave_multi_return",
//!     "inputs": [
//!         {
//!             "method": "step_leave_multi_return",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1", "10"
//!     ]
//! }, {
//!     "name": "step_function_definition",
//!     "inputs": [
//!         {
//!             "method": "step_function_definition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_function_definition_leave",
//!     "inputs": [
//!         {
//!             "method": "step_function_definition_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "step_no_mutation",
//!     "inputs": [
//!         {
//!             "method": "step_no_mutation",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "15", "5"
//!     ]
//! }, {
//!     "name": "init_let_multi",
//!     "inputs": [
//!         {
//!             "method": "init_let_multi",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "2"
//!     ]
//! }, {
//!     "name": "init_if_leave",
//!     "inputs": [
//!         {
//!             "method": "init_if_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "42"
//!     ]
//! }, {
//!     "name": "init_leave",
//!     "inputs": [
//!         {
//!             "method": "init_leave",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "7"
//!     ]
//! }, {
//!     "name": "init_switch",
//!     "inputs": [
//!         {
//!             "method": "init_switch",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "2"
//!     ]
//! }, {
//!     "name": "init_nested_for_break_continue",
//!     "inputs": [
//!         {
//!             "method": "init_nested_for_break_continue",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "3"
//!     ]
//! }, {
//!     "name": "init_block",
//!     "inputs": [
//!         {
//!             "method": "init_block",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "4"
//!     ]
//! }, {
//!     "name": "init_function_call",
//!     "inputs": [
//!         {
//!             "method": "init_function_call",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0;

contract Test {
    function step_empty() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { } { i := add(i, 1) y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_assignment() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) } { y := add(y, i) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_multi_assignment() external pure {
        assembly {
            function f(a, b) -> c, d { c := add(a, 1) d := add(b, a) }
            function h(m) -> y { for { let i := 0 } lt(i, m) { i, y := f(i, y) } { } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_builtin_call() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) mstore(0x20, add(mload(0x20), i)) } { } y := mload(0x20) }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_function_call() external pure {
        assembly {
            function bump() { mstore(0x20, add(mload(0x20), 1)) }
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) bump() } { } y := mload(0x20) }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_let() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { let d := 2 i := add(i, d) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_let_uninit() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { let d i := add(i, add(d, 1)) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_let_multi() external pure {
        assembly {
            function two() -> a, b { a := 1 b := 2 }
            function h(m) -> y { for { let i := 0 } lt(i, m) { let a, b := two() i := add(i, add(a, b)) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_if() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) if eq(i, 3) { i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_if_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) if eq(i, 3) { leave } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_switch_default() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { switch i case 0 { i := 2 } default { i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_switch_no_default() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) switch i case 2 { i := 3 } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_switch_only_default() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { switch i default { i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_switch_case_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) switch i case 2 { y := add(y, 100) leave } default { } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_switch_default_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { i := add(i, 1) switch i case 1 { } default { y := add(y, 100) leave } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_nested_for() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { for { let j := 0 } lt(j, 3) { j := add(j, 1) } { i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_nested_for_break_continue() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { for { let j := 0 } lt(j, 10) { j := add(j, 1) } { if eq(j, 1) { continue } if eq(j, 3) { break } i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_nested_for_step_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { for { let j := 0 } lt(j, 2) { y := add(y, 100) leave } { i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_nested_for_body_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { for { let j := 0 } lt(j, 2) { j := add(j, 1) } { if eq(i, 3) { y := add(y, 100) leave } i := add(i, 1) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_nested_for_then_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { for { } lt(i, 2) { i := add(i, 1) } { y := add(y, 10) } y := add(y, 100) leave } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_block() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { { let d := 1 i := add(i, d) } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_block_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { { i := add(i, 1) if eq(i, 2) { y := add(y, 100) leave } } } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { y := add(y, 100) leave } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_leave_unreachable_tail() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { leave i := add(i, 1) y := add(y, 100) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_leave_multi_return() external pure {
        assembly {
            function h(m) -> y, z { for { let i := 0 } lt(i, m) { z := add(z, 10) leave } { y := add(y, 1) } }
            let a, b := h(5)
            mstore(0, a)
            mstore(0x20, b)
            return(0, 64)
        }
    }

    function step_function_definition() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { function inc(a) -> b { b := add(a, 1) } i := inc(i) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_function_definition_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { function inc(a) -> b { b := add(a, 1) leave b := 0 } i := inc(i) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function step_no_mutation() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 } lt(i, m) { mstore(0x20, add(mload(0x20), 1)) } { i := add(i, 1) y := add(y, i) } }
            mstore(0, h(5))
            return(0, 64)
        }
    }

    function init_let_multi() external pure {
        assembly {
            function two() -> a, b { a := 1 b := 2 }
            function h(m) -> y { for { let a, b := two() let i := a } lt(i, m) { i := add(i, b) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function init_if_leave() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 if eq(m, 5) { y := 42 leave } } lt(i, m) { i := add(i, 1) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function init_leave() external pure {
        assembly {
            function h(m) -> y { y := 7 for { leave } 1 { } { y := 0 } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function init_switch() external pure {
        assembly {
            function h(m) -> y { for { let i switch m case 5 { i := 3 } default { i := 0 } } lt(i, m) { i := add(i, 1) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function init_nested_for_break_continue() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 for { let j := 0 } lt(j, 10) { j := add(j, 1) } { if eq(j, 1) { continue } if eq(j, 3) { break } i := add(i, 1) } } lt(i, m) { i := add(i, 1) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function init_block() external pure {
        assembly {
            function h(m) -> y { for { let i := 0 { i := add(i, 1) } } lt(i, m) { i := add(i, 1) } { y := add(y, 1) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }

    function init_function_call() external pure {
        assembly {
            function bump() { mstore(0x20, add(mload(0x20), 1)) }
            function h(m) -> y { for { let i := 0 bump() } lt(i, m) { i := add(i, 1) } { y := add(y, mload(0x20)) } }
            mstore(0, h(5))
            return(0, 32)
        }
    }
}
