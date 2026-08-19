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
//!         "1"
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
//!         "19"
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
//!         "32"
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
//!         "2383"
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
//!         "0xF100"
//!     ]
//! }, {
//!     "name": "initial",
//!     "inputs": [
//!         {
//!             "method": "initial",
//!             "calldata": [
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "overwrite_smaller",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "300", "200"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "200"
//!     ]
//! }, {
//!     "name": "overwrite_equals",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "1233", "1233"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1233"
//!     ]
//! }, {
//!     "name": "overwrite_bigger",
//!     "inputs": [
//!         {
//!             "method": "overwrite",
//!             "calldata": [
//!                 "0", "5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0;

contract Test {
    function main(uint256 len) external returns(uint256 result) {
        assembly {
            mstore(0, len)
            pop(call(gas(), address(), 0, 0, 32, 0, 0))
            result := returndatasize()
        }
    }

    function initial() external returns(uint256 result) {
        assembly {
            result := returndatasize()
        }
    }

    function overwrite(uint256 len1, uint256 len2) external returns(uint256 result) {
        assembly {
            mstore(0, len1)
            pop(call(gas(), address(), 0, 0, 32, 0, 0))
            mstore(0, len2)
            pop(call(gas(), address(), 0, 0, 32, 0, 0))
            result := returndatasize()
        }
    }

    fallback() external {
        assembly {
            let len := calldataload(0)
            return(0, len)
        }
    }
}
