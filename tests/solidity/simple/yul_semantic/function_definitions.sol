//! { "cases": [ {
//!     "name": "no_args_and_ret",
//!     "inputs": [
//!         {
//!             "method": "no_args_and_ret",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "many_args",
//!     "inputs": [
//!         {
//!             "method": "many_args",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "120"
//!     ]
//! }, {
//!     "name": "many_ret_values",
//!     "inputs": [
//!         {
//!             "method": "many_ret_values",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "78"
//!     ]
//! },  {
//!     "name": "empty_body_arg_side_effect",
//!     "inputs": [
//!         {
//!             "method": "empty_body_arg_side_effect",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "leave_in_function",
//!     "inputs": [
//!         {
//!             "method": "leave_in_function",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef01" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "return_in_function",
//!     "inputs": [
//!         {
//!             "method": "return_in_function",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [ "0xdeadbeef01" ],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef01" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "revert_in_function",
//!     "inputs": [
//!         {
//!             "method": "revert_in_function",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [ "0xdeadbeef01" ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "default_ret_value",
//!     "inputs": [
//!         {
//!             "method": "default_ret_value",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "sibling_scopes",
//!     "inputs": [
//!         {
//!             "method": "sibling_scopes",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1", "2"
//!     ]
//! }, {
//!     "name": "nested_definition",
//!     "inputs": [
//!         {
//!             "method": "nested_definition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "16"
//!     ]
//! }, {
//!     "name": "forward_reference",
//!     "inputs": [
//!         {
//!             "method": "forward_reference",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "107"
//!     ]
//! }, {
//!     "name": "leave_in_for_step",
//!     "inputs": [
//!         {
//!             "method": "leave_in_for_step",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0;

contract Test {
    function no_args_and_ret() external {
        assembly {
            function $_() {
                mstore(0, 0xdeadbeef)
                log0(0, 32)
            }
            $_()
        }
    }

    function many_args() external pure {
        assembly {
            function f(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15) -> ret {
                ret := add(
                    add(
                        add(
                            add(_1, _2),
                            add(_3, _4)
                        ),
                        add(
                            add(_5, _6),
                            add(_7, _8)
                        )
                    ),
                    add(
                        add(
                            add(_9, _10),
                            add(_11, _12)
                        ),
                        add(
                            add(_13, _14),
                            _15
                        )
                    )
                )
            }
            mstore(0, f(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15))
            return(0, 32)
        }
    }

    function many_ret_values() external pure {
        assembly {
            function f() -> _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12 {
                _1 := 1
                _2 := 2
                _3 := 3
                _4 := 4
                _5 := 5
                _6 := 6
                _7 := 7
                _8 := 8
                _9 := 9
                _10 := 10
                _11 := 11
                _12 := 12
            }
            let _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12 := f()
            mstore(
                0,
                add(
                    add(
                        add(
                            add(_1, _2),
                            add(_3, _4)
                        ),
                        add(
                            add(_5, _6),
                            add(_7, _8)
                        )
                    ),
                    add(
                        add(_9, _10),
                        add(_11, _12)
                    )
                )
            )
            return(0, 32)
        }
    }

    function empty_body_arg_side_effect() external {
        assembly {
            function f() -> ret {
                mstore(0, 0xdeadbeef)
                log0(0, 32)
            }
            function g(a) {}
            g(f())
        }
    }

    function leave_in_function() external {
        assembly {
            function f() {
                mstore(0, 0xdeadbeef01)
                log0(0, 32)
                leave
                mstore(0, 0xdeadbeef02)
                log0(0, 32)
            }
            f()
        }
    }

    function return_in_function() external {
        assembly {
            function f() {
                mstore(0, 0xdeadbeef01)
                log0(0, 32)
                return(0, 32)
                mstore(0, 0xdeadbeef02)
                log0(0, 32)
            }
            f()
        }
    }

    function revert_in_function() external {
        assembly {
            function f() {
                mstore(0, 0xdeadbeef01)
                log0(0, 32)
                revert(0, 32)
                mstore(0, 0xdeadbeef02)
                log0(0, 32)
            }
            f()
        }
    }

    function default_ret_value() external pure {
        assembly {
            function f() -> ret {}
            mstore(0, f())
            return(0, 32)
        }
    }

    function sibling_scopes() external pure {
        assembly {
            {
                function f() -> ret { ret := 1 }
                mstore(0, f())
            }
            {
                function f() -> ret { ret := 2 }
                mstore(32, f())
            }
            return(0, 64)
        }
    }

    function nested_definition() external pure {
        assembly {
            function outer(x) -> ret {
                function inner(y) -> z { z := mul(y, 3) }
                ret := add(inner(x), 1)
            }
            mstore(0, outer(5))
            return(0, 32)
        }
    }

    function forward_reference() external pure {
        assembly {
            mstore(0, later(7))
            function later(x) -> ret { ret := add(x, 100) }
            return(0, 32)
        }
    }

    function leave_in_for_step() external pure {
        assembly {
            function h(m) -> y {
                for { let i := 0 } lt(i, m) { leave } { y := add(y, 1) }
            }
            mstore(0, h(5))
            return(0, 32)
        }
    }
}
