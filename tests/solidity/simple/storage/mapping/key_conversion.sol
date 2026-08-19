//! { "cases": [ {
//!     "name": "negative_narrow_key",
//!     "inputs": [
//!         {
//!             "method": "setNarrow",
//!             "calldata": [
//!                 "-5", "42"
//!             ]
//!         }, {
//!             "method": "getWide",
//!             "calldata": [
//!                 "-5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "42"
//!     ]
//! }, {
//!     "name": "positive_narrow_key",
//!     "inputs": [
//!         {
//!             "method": "setNarrow",
//!             "calldata": [
//!                 "7", "13"
//!             ]
//!         }, {
//!             "method": "getWide",
//!             "calldata": [
//!                 "7"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "13"
//!     ]
//! }, {
//!     "name": "literal_word_key",
//!     "inputs": [
//!         {
//!             "method": "setLiteral",
//!             "calldata": [
//!                 "42"
//!             ]
//!         }, {
//!             "method": "getWord",
//!             "calldata": [
//!                 "0x6162630000000000000000000000000000000000000000000000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "42"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    mapping(int256 => uint256) signedMap;
    mapping(bytes32 => uint256) wordMap;

    function setNarrow(int8 key, uint256 value) public {
        signedMap[key] = value;
    }

    function getWide(int256 key) public view returns (uint256) {
        return signedMap[key];
    }

    function setLiteral(uint256 value) public {
        wordMap["abc"] = value;
    }

    function getWord(bytes32 key) public view returns (uint256) {
        return wordMap[key];
    }
}
