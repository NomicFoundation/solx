//! { "cases": [ {
//!     "name": "zero_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "17", "233", "300", "0xAA", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "32", "128", "160", "0xFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "small_ordinary_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "19", "0", "32", "33", "0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "word_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "272", "17", "500", "0xDE", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "2383", "122", "2383", "2560", "0x44", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_words_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "64", "100", "164", "0xFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "big_ordinary_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFAA", "32", "32", "0", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_64_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFC0", "322", "333", "10", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_63_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFC1", "0", "512", "4", "0xBB", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_between_32_64_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFCF", "1", "1", "52", "52", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_33_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFDF", "0", "1", "48", "0xC8", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE0", "55", "64", "32", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFE1", "1", "17", "3", "3", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFEF", "5", "1", "128", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFF", "17", "55", "222", "0xAA", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x100000000", "256", "32", "256", "12", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "3334353453452342342354355544445321191012012", "32", "1", "1", "0x55", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_to",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "100", "96", "0", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "17", "0", "233", "300", "0xAA", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "1", "128", "160", "0xFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "small_ordinary_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "19", "32", "51", "0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "word_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "272", "32", "17", "500", "0xDE", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "122", "2383", "2383", "5800", "0x44", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_words_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "64", "0xF100", "100", "0xF180", "0xFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "big_ordinary_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "0xFFFFFFAA", "32", "0", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_64_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "322", "0xFFFFFFC0", "333", "10", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_63_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFC1", "512", "4", "0xBB", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_between_32_64_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFCF", "1", "52", "52", "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_33_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0xFFFFFFDF", "1", "48", "0xC8", "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "55", "0xFFFFFFE0", "64", "32", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0xFFFFFFE1", "17", "3", "3", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "5", "0xFFFFFFEF", "1", "128", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "17", "0xFFFFFFFF", "55", "222", "0xAA", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "0x100000000", "32", "256", "12", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "3334353453452342342354355544445321191012012", "1", "1", "0x55", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_from",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "100", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "96", "0", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "zero_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "17", "233", "0", "300", "0xAA", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "one_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "128", "32", "160", "0xFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "small_ordinary_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "32", "19", "51", "0", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "word_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "272", "17", "32", "500", "0xDE", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "122", "2383", "2383", "5800", "0x44", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_words_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "64", "100", "0xF100", "0xF180", "0xFF", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "big_ordinary_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "32", "0xFFFFFFAA", "0", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_64_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "322", "333", "0xFFFFFFC0", "10", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_63_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "512", "0xFFFFFFC1", "4", "0xBB", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_between_32_64_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "1", "0xFFFFFFCF", "52", "52", "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_33_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "1", "0xFFFFFFDF", "48", "0xC8", "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_32_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "55", "64", "0xFFFFFFE0", "32", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_31_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "17", "0xFFFFFFE1", "3", "3", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_ordinary_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "5", "1", "0xFFFFFFEF", "128", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_minus_one_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "17", "55", "0xFFFFFFFF", "222", "0xAA", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "2_in_32_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "256", "32", "0x100000000", "256", "12", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "ordinary_bigger_2_in_32_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "1", "3334353453452342342354355544445321191012012", "1", "0x55", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "max_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "100", "96", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0", "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "too_small_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "64", "100", "100", "199", "0xFF", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "big_from_zero_size",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "32", "45678765435678987654323456789834765434567890", "0", "32", "0xAA", "1"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "initial",
//!     "inputs": [
//!         {
//!             "method": "initial",
//!             "calldata": [
//!                 "222", "12", "100"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! }, {
//!     "name": "overwrite_bigger",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "32", "1", "128",
//!                 "117", "0xAA",
//!                 "256", "0xFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "overwrite_same_size",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "32", "0", "32",
//!                 "33", "1",
//!                 "33", "0xAB"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "overwrite_smaller",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "0", "20", "128",
//!                 "160", "0xFF",
//!                 "147", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "exception": true,
//!         "return_data": []
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0;

contract Test {
    function main(uint256 to, uint256 from, uint256 len, uint256 returndata_len, uint8 val, bool check) external returns(uint256, uint256, uint256) {
        assembly {
            mstore(0, returndata_len)
            mstore(32, val)
            pop(call(gas(), address(), 0, 0, 64, 0, 0))

            returndatacopy(to, from, len)

            if gt(check, 0) {
                for { let i := 0 } lt(i, len) { i := add(i, 1) }
                {
                    if iszero(eq(
                        shr(248, mload(add(to, i))),
                        val
                    )) {
                        mstore(0, 0)
                        return(0, 32)
                    }
                }
            }

            mstore(0, 1)
            return(0, 32)
        }
    }

    function initial(uint256 to, uint256 from, uint256 len) external returns(uint256 result) {
        assembly {
            returndatacopy(to, from, len)

            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                if iszero(eq(
                    shr(248, mload(add(to, i))),
                    0
                )) {
                    mstore(0, 0)
                    return(0, 32)
                }
            }

            mstore(0, 1)
            return(0, 32)
        }
    }

    function overwrite(
        uint256 to,
        uint256 from,
        uint256 len,

        uint256 returndata_len1,
        uint8 val1,

        uint256 returndata_len2,
        uint8 val2
    ) external returns(uint256 result) {
        assembly {
            mstore(0, returndata_len1)
            mstore(32, val1)
            pop(call(gas(), address(), 0, 0, 64, 0, 0))


            mstore(0, returndata_len2)
            mstore(32, val2)
            pop(call(gas(), address(), 0, 0, 64, 0, 0))

            returndatacopy(to, from, len)

            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                if iszero(eq(
                    shr(248, mload(add(to, i))),
                    val2
                )) {
                    mstore(0, 0)
                    return(0, 32)
                }
            }

            mstore(0, 1)
            return(0, 32)
        }
    }

    fallback() external {
        assembly {
            let len := calldataload(0)
            let val := calldataload(32)
            for { let i := 0 } lt(i, len) { i := add(i, 1) }
            {
                mstore8(i, val)
            }
            return(0, len)
        }
    }
}