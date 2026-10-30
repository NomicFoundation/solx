//! { "cases": [ {
//!     "name": "empty",
//!     "inputs": [ { "method": "pair", "calldata": [ "0" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "one_byte_short",
//!     "inputs": [ { "method": "pair", "calldata": [ "63" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "exact",
//!     "inputs": [ { "method": "pair", "calldata": [ "64" ] } ],
//!     "expected": [ "64", "65" ]
//! }, {
//!     "name": "longer",
//!     "inputs": [ { "method": "pair", "calldata": [ "96" ] } ],
//!     "expected": [ "96", "97" ]
//! }, {
//!     "name": "try_short",
//!     "inputs": [ { "method": "try_pair", "calldata": [ "32" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "try_exact",
//!     "inputs": [ { "method": "try_pair", "calldata": [ "64" ] } ],
//!     "expected": [ "1", "64", "65" ]
//! }, {
//!     "name": "single_short",
//!     "inputs": [ { "method": "single", "calldata": [ "31" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "single_exact",
//!     "inputs": [ { "method": "single", "calldata": [ "32" ] } ],
//!     "expected": [ "32" ]
//! }, {
//!     "name": "none_empty",
//!     "inputs": [ { "method": "none", "calldata": [ "0" ] } ],
//!     "expected": [ "1" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

interface Raw {
    function raw(uint256 n) external returns (uint256, uint256);
    function rawOne(uint256 n) external returns (uint256);
    function rawNone(uint256 n) external;
}

// External calls whose results have a static size: return data shorter than
// that reverts the caller, longer is ignored.
contract Test {
    // Returns n bytes: the words n, n + 1, ...
    function raw(uint256 n) external pure {
        assembly {
            for { let i := 0 } lt(mul(i, 32), n) { i := add(i, 1) } {
                mstore(mul(i, 32), add(n, i))
            }
            return(0, n)
        }
    }

    function rawOne(uint256 n) external pure {
        assembly {
            mstore(0, n)
            return(0, n)
        }
    }

    function rawNone(uint256 n) external pure {
        assembly {
            return(0, n)
        }
    }

    function pair(uint256 n) external returns (uint256, uint256) {
        return Raw(address(this)).raw(n);
    }

    function try_pair(uint256 n) external returns (bool, uint256, uint256) {
        try Raw(address(this)).raw(n) returns (uint256 a, uint256 b) {
            return (true, a, b);
        } catch {
            return (false, 0, 0);
        }
    }

    function single(uint256 n) external returns (uint256) {
        return Raw(address(this)).rawOne(n);
    }

    function none(uint256 n) external returns (bool) {
        Raw(address(this)).rawNone(n);
        return true;
    }
}
