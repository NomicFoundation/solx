//! { "cases": [ {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "124"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x7C00000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "232322334234234234234"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x7A00000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "19", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFF00000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x0100000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "2383", "23823848238483284838488483848348838348433"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x9100000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "-12121312312312"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x0800000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "9389324993493993"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "12"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "9821984989898348938948943983489893489389438349834"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "999999999999"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "111"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "453245432"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "33"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "2323", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFF00FF0000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "one_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "4", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFF01FF0000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_small_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "0", "197", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xC5FF000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "max_byte_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "1233", "255", "0x00"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x00FF000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! },  {
//!     "name": "256_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "1233", "256", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFF00FF0000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_big_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "12122", "13482894893289489238489829389489238489284892388292332", "0xAA"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xAAECAA0000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "max_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "1024", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x00FF000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 key, uint256 value) external pure returns(uint256) {
        assembly {
            mstore8(key, value)
            return(key, 8)
        }
    }

    function overwrite(uint256 key, uint256 value, uint8 beforeValue) external pure returns(uint256) {
        assembly {
            let offset := sub(key, gt(key, 0))
            let len := add(2, gt(key, 0))
            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                mstore8(add(offset, i), beforeValue)
            }
            mstore8(key, value)
            return(offset, len)
        }
    }
}
