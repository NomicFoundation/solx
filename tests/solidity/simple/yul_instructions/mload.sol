//! { "cases": [ {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "19"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "word",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "2383"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "big_ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA"
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
//!                 "0xFFFFFFE0"
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
//!                 "0xFFFFFFE1"
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
//!                 "0xFFFFFFEF"
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
//!                 "0xFFFFFFFF"
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
//!                 "0x100000000"
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
//!                 "3334353453452342342354355544445321191012012"
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
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_value_zero",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_value_one",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_value_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "19", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_value_word",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "32", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_value_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "2383", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_value_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0xF100", "0", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_value_zero",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_value_one",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_value_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "19", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_value_word",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "32", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_value_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "2383", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_value_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0xF100", "1", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_value_zero",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "2513561237612763712737127371273871283871273172", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2513561237612763712737127371273871283871273172"
//!     ]
//! }, {
//!     "name": "ordinary_value_one",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "2513561237612763712737127371273871283871273172", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2513561237612763712737127371273871283871273172"
//!     ]
//! }, {
//!     "name": "ordinary_value_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "19", "2513561237612763712737127371273871283871273172", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2513561237612763712737127371273871283871273172"
//!     ]
//! }, {
//!     "name": "ordinary_value_word",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "32", "2513561237612763712737127371273871283871273172", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2513561237612763712737127371273871283871273172"
//!     ]
//! }, {
//!     "name": "ordinary_value_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "2383", "2513561237612763712737127371273871283871273172", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2513561237612763712737127371273871283871273172"
//!     ]
//! }, {
//!     "name": "ordinary_value_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0xF100", "2513561237612763712737127371273871283871273172", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2513561237612763712737127371273871283871273172"
//!     ]
//! }, {
//!     "name": "max_value_zero",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "max_value_one",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "max_value_small_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "19", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "max_value_word",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "32", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "max_value_ordinary",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "2383", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! }, {
//!     "name": "max_value_ordinary_words",
//!     "inputs": [
//!         {
//!             "method": "with_value",
//!             "calldata": [
//!                 "0xF100", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.19;

contract Test {
    function main(uint256 key) external pure returns(uint256) {
        assembly {
            mstore(0, mload(key))
            return(0, 32)
        }
    }

    function with_value(uint256 key, uint256 value, uint8 outside) external pure returns(uint256) {
        assembly {
            if gt(key, 0) {
                mstore8(sub(key, 1), outside)
            }
            mstore8(add(key, 32), outside)
            mstore(key, value)
            mstore(0, mload(key))
            return(0, 32)
        }
    }
}
