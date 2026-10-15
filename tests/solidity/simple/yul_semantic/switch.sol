//! { "cases": [ {
//!     "name": "true_condition",
//!     "inputs": [
//!         {
//!             "method": "true_condition",
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
//!     "name": "false_condition",
//!     "inputs": [
//!         {
//!             "method": "false_condition",
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
//!     "name": "string_condition",
//!     "inputs": [
//!         {
//!             "method": "string_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef02" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "empty_string_condition",
//!     "inputs": [
//!         {
//!             "method": "empty_string_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef03" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "zero_condition",
//!     "inputs": [
//!         {
//!             "method": "zero_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef04" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "one_condition",
//!     "inputs": [
//!         {
//!             "method": "one_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef05" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "number_condition",
//!     "inputs": [
//!         {
//!             "method": "number_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef06" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "runtime_condition",
//!     "inputs": [
//!         {
//!             "method": "runtime_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef07" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "return_in_condition",
//!     "inputs": [
//!         {
//!             "method": "return_in_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef08"
//!     ]
//! }, {
//!     "name": "revert_in_condition",
//!     "inputs": [
//!         {
//!             "method": "revert_in_condition",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": [ "0xdeadbeef09" ]
//!     }
//! }, {
//!     "name": "return_in_case",
//!     "inputs": [
//!         {
//!             "method": "return_in_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef0a"
//!     ]
//! }, {
//!     "name": "revert_in_case",
//!     "inputs": [
//!         {
//!             "method": "revert_in_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": [ "0xdeadbeef0b" ]
//!     }
//! }, {
//!     "name": "nested_switchs",
//!     "inputs": [
//!         {
//!             "method": "nested_switchs",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef0c"
//!     ]
//! }, {
//!     "name": "one_reachable_case",
//!     "inputs": [
//!         {
//!             "method": "one_reachable_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef0d"
//!     ]
//! }, {
//!     "name": "one_unreachable_case",
//!     "inputs": [
//!         {
//!             "method": "one_unreachable_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef0e"
//!     ]
//! }, {
//!     "name": "only_default_case",
//!     "inputs": [
//!         {
//!             "method": "only_default_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef0f"
//!     ]
//! }, {
//!     "name": "reachable_default_case",
//!     "inputs": [
//!         {
//!             "method": "reachable_default_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef10"
//!     ]
//! }, {
//!     "name": "unreachable_default_case",
//!     "inputs": [
//!         {
//!             "method": "unreachable_default_case",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef11"
//!     ]
//! }, {
//!     "name": "empty_default_case_condition_side_effect",
//!     "inputs": [
//!         {
//!             "method": "empty_default_case_condition_side_effect",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef12"
//!     ]
//! }, {
//!     "name": "many_cases",
//!     "inputs": [
//!         {
//!             "method": "many_cases",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef13"
//!     ]
//! }, {
//!     "name": "complex",
//!     "inputs": [
//!         {
//!             "method": "complex",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [ "0xdeadbeef04" ],
//!         "events": [
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef02" ]
//!             },
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef03" ]
//!             },
//!             {
//!                 "topics": [],
//!                 "values": [ "0xdeadbeef04" ]
//!             }
//!         ]
//!     }
//! }, {
//!     "name": "wide_case_zero",
//!     "inputs": [
//!         {
//!             "method": "wide_case",
//!             "calldata": [ "0" ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef14"
//!     ]
//! }, {
//!     "name": "wide_case_two_pow_63",
//!     "inputs": [
//!         {
//!             "method": "wide_case",
//!             "calldata": [ "0x8000000000000000" ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef15"
//!     ]
//! }, {
//!     "name": "wide_case_two_pow_255",
//!     "inputs": [
//!         {
//!             "method": "wide_case",
//!             "calldata": [ "0x8000000000000000000000000000000000000000000000000000000000000000" ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef16"
//!     ]
//! }, {
//!     "name": "wide_case_default",
//!     "inputs": [
//!         {
//!             "method": "wide_case",
//!             "calldata": [ "1" ]
//!         }
//!     ],
//!     "expected": [
//!         "0xdeadbeef17"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.6.2;

contract Test {
    function true_condition() external {
        assembly {
            switch true
                case 12312 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 1 {
                    mstore(0, 0xdeadbeef)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function false_condition() external {
        assembly {
            switch false
                case 1 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 0 {
                    mstore(0, 0xdeadbeef01)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function string_condition() external {
        assembly {
            switch "abc"
                case "acd" {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 44048180597813453602326562734351324025098966208897425494240603688123167145984 {
                    mstore(0, 0xdeadbeef02)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function empty_string_condition() external {
        assembly {
            switch ""
                case "0" {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 0 {
                    mstore(0, 0xdeadbeef03)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function zero_condition() external {
        assembly {
            switch 0
                case true {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case false {
                    mstore(0, 0xdeadbeef04)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function one_condition() external {
        assembly {
            switch 1
                case 0x10 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 1 {
                    mstore(0, 0xdeadbeef05)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function number_condition() external {
        assembly {
            switch 344342323832832883
                case 344342323832832884 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 344342323832832883 {
                    mstore(0, 0xdeadbeef06)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function runtime_condition() external payable {
        assembly {
            switch add(callvalue(), 100)
                case 212 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 100 {
                    mstore(0, 0xdeadbeef07)
                    log0(0, 32)
                }
            return(0, 0)
        }
    }

    function return_in_condition() external pure {
        assembly {
            function f() -> ret {
                mstore(0, 0xdeadbeef08)
                ret := 1
                return(0, 32)
            }
            switch f()
                case 0 {
                    mstore(0, 0xdeadbeef)
                }
            revert(0, 32)
        }
    }

    function revert_in_condition() external pure {
        assembly {
            function f() -> ret {
                mstore(0, 0xdeadbeef09)
                revert(0, 32)
            }
            switch f()
                default {
                    mstore(0, 0xdeadbeef)
                }
            return(0, 32)
        }
    }

    function return_in_case() external {
        assembly {
            switch 4
                case 4  {
                    mstore(0, 0xdeadbeef0a)
                    return(0, 32)
                }
                case 5 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
            revert(0, 32)
        }
    }

    function revert_in_case() external {
        assembly {
            switch 100
                case 0 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 100 {
                    mstore(0, 0xdeadbeef0b)
                    revert(0, 32)
                }
                default {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
            return(0, 32)
        }
    }

    function nested_switchs() external {
        assembly {
            switch true
                case 1 {
                    switch 10
                        case 10 {
                            mstore(0, 0xdeadbeef0c)
                        }
                        case 11 {
                            mstore(0, 0xe2202)
                            log0(0, 32)
                        }
                }
                default {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
            return(0, 32)
        }
    }

    function one_reachable_case() external pure {
        assembly {
            switch 100
                case 100 {
                    mstore(0, 0xdeadbeef0d)
                }
            return(0, 32)
        }
    }

    function one_unreachable_case() external pure {
        assembly {
            mstore(0, 0xdeadbeef0e)
            switch 10
                case 11 {
                    mstore(0, 0xe2202)
                }
            return(0, 32)
        }
    }

    function only_default_case() external pure {
        assembly {
            switch "a"
                default {
                    mstore(0, 0xdeadbeef0f)
                }
            return(0, 32)
        }
    }

    function reachable_default_case() external {
        assembly {
            switch 1231233123
                case 0 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case "a" {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                default {
                    mstore(0, 0xdeadbeef10)
                }
            return(0, 32)
        }
    }

    function unreachable_default_case() external {
        assembly {
            switch 49
                case 0 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 49 {
                    mstore(0, 0xdeadbeef11)
                }
                case 222 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                default {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
            return(0, 32)
        }
    }

    function empty_default_case_condition_side_effect() external pure {
        assembly {
            function f() -> ret {
                mstore(0, 0xdeadbeef12)
            }
            switch f()
            default {}
            return(0, 32)
        }
    }

    function many_cases() external {
        assembly {
            switch 100
                case 0 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 20 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 40 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 60 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 80 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 100 {
                    mstore(0, 0xdeadbeef13)
                }
                case 120 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 140 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 160 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
                case 180 {
                    mstore(0, 0xe2202)
                    log0(0, 32)
                }
            return(0, 32)
        }
    }

    function complex() external payable {
        assembly {
            switch eq(callvalue(), 0)
                case 1 {
                    switch "a"
                        case 0x6200000000000000000000000000000000000000000000000000000000000000 {
                            mstore(0, 0xdeadbeef01)
                            log0(0, 32)
                        }
                        default {
                            mstore(0, 0xdeadbeef02)
                            log0(0, 32)
                            switch 5
                                case 5 {
                                    mstore(0, 0xdeadbeef03)
                                    log0(0, 32)
                                }
                                default {
                                    revert(0, 32)
                                }
                        }
                    mstore(0, 0xdeadbeef04)
                    log0(0, 32)
                    switch add(1, 2)
                        case 1 {
                            revert(0, 32)
                        }
                        case 2 {
                            revert(0, 32)
                        }
                }
                default {
                    switch 5
                        case 5 {
                            mstore(0, 0xdeadbeef05)
                            log0(0, 32)
                        }
                        case 4 {
                            mstore(0, 0xdeadbeef06)
                            log0(0, 32)
                            revert(0, 32)
                        }
                }
            return(0, 32)
        }
    }

    function wide_case(uint a) external pure returns (uint r) {
        assembly {
            switch a
                case 0 {
                    r := 0xdeadbeef14
                }
                case 0x8000000000000000 {
                    r := 0xdeadbeef15
                }
                case 0x8000000000000000000000000000000000000000000000000000000000000000 {
                    r := 0xdeadbeef16
                }
                default {
                    r := 0xdeadbeef17
                }
        }
    }
}
