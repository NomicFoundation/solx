//! { "modes": [
//!     "Y-",
//!     "E-",
//!     "I-",
//!     "E+ 0.4",
//!     "E+ 0.5"
//! ], "cases": [  {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "322332232323"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "96"
//!     ]
//! }, {
//!     "name": "one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "96"
//!     ]
//! }, {
//!     "name": "small_ordinar",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "19", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "96"
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
//!         "96"
//!     ]
//! }, {
//!     "name": "ordinar",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "2383", "12123124235436554647467567567576"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2432"
//!     ]
//! }, {
//!     "name": "ordinar_words",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xF100", "2343355345423423423"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xF120"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

// For small numbers, msize will be 96, as solidity writes free memory pointer
contract Test {
    function main(uint256 key, uint256 value) external pure returns(uint256 result) {
        assembly {
            mstore(key, value)
            result := msize()
        }
    }
}
