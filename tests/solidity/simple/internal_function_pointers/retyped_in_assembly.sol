//! { "cases": [ {
//!     "name": "address",
//!     "inputs": [
//!         {
//!             "method": "compareAddresses",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "bytes32",
//!     "inputs": [
//!         {
//!             "method": "compareBytes32",
//!             "calldata": [ "1", "2" ]
//!         }
//!     ],
//!     "expected": [ "1" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// The cast OpenZeppelin's Arrays.sort applies to address and bytes32 comparators.
contract Test {
    function _lt(address a, address b) private pure returns (bool) {
        return a < b;
    }

    function _ltBytes32(bytes32 a, bytes32 b) private pure returns (bool) {
        return a < b;
    }

    function _castToUint256Comp(
        function(address, address) pure returns (bool) input
    ) private pure returns (function(uint256, uint256) pure returns (bool) output) {
        assembly {
            output := input
        }
    }

    function _castToUint256CompBytes32(
        function(bytes32, bytes32) pure returns (bool) input
    ) private pure returns (function(uint256, uint256) pure returns (bool) output) {
        assembly {
            output := input
        }
    }

    function compareAddresses() public pure returns (bool) {
        function(uint256, uint256) pure returns (bool) comp = _castToUint256Comp(_lt);
        return comp(1, 2) && !comp(2, 1);
    }

    function compareBytes32(uint256 a, uint256 b) public pure returns (bool) {
        return _castToUint256CompBytes32(_ltBytes32)(a, b);
    }
}
