//! { "cases": [ {
//!     "name": "zero_by_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [],
//!         "exception": true
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "ordinary_by_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [],
//!         "exception": true
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "min_by_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-128", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [],
//!         "exception": true
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "max_by_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "127", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [],
//!         "exception": true
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "ordinary_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "6"
//!     ]
//! }, {
//!     "name": "ordinary_negative_divided",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42", "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-6"
//!     ]
//! }, {
//!     "name": "ordinary_negative_divider",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "-7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-6"
//!     ]
//! }, {
//!     "name": "ordinary_negative_both",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42", "-7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "6"
//!     ]
//! }, {
//!     "name": "ordinary_max_remainder",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "48", "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "6"
//!     ]
//! }, {
//!     "name": "ordinary_next",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "49", "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "7"
//!     ]
//! }, {
//!     "name": "to_min_from_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "127", "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-127"
//!     ]
//! }, {
//!     "name": "to_min_from_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-128", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-128"
//!     ]
//! }, {
//!     "name": "to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "127", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "127"
//!     ]
//! }, {
//!     "name": "to_one_positives",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "127", "127"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "to_one_negatives",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-128", "-128"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "min_to_negative_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-127", "3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-42"
//!     ]
//! }, {
//!     "name": "max_to_negative_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "127", "-3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-42"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(int8 a, int8 b) public pure returns(int8) {
        int8 c = a;
        c /= b;
        return c;
    }
}
