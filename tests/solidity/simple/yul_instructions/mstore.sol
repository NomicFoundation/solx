//! { "cases": [ {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "394934348834929392931283812381292121"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "394934348834929392931283812381292121"
//!     ]
//! }, {
//!     "name": "one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "23232"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "23232"
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
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
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
//!         "1"
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
//!         "23823848238483284838488483848348838348433"
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
//!         "-12121312312312"
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
//!         "0xFF00000000000000000000000000000000000000000000000000000000000000", "0x00FF000000000000000000000000000000000000000000000000000000000000"
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
//!         "0xFF00000000000000000000000000000000000000000000000000000000000000", "0x01FF000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_value",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "0", "13482894893289489238489829389489238489284892388292332", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x0000000000000000000024095EFD136928AD0DD5540CA95A20796621785CF6EC", "0xFF00000000000000000000000000000000000000000000000000000000000000"
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
//!         "0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFF00000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 key, uint256 value) external pure returns(uint256) {
        assembly {
            mstore(key, value)
            return(key, 32)
        }
    }

    function overwrite(uint256 key, uint256 value, uint8 beforeValue) external pure returns(uint256) {
        assembly {
            let offset := sub(key, gt(key, 0))
            let len := add(33, gt(key, 0))
            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                mstore8(add(offset, i), beforeValue)
            }
            mstore(key, value)
            return(offset, len)
        }
    }
}
