//! { "modes": [ "E" ], "cases": [ {
//!     "name": "storage_signature",
//!     "inputs": [ { "method": "storageSignature", "calldata": [] } ],
//!     "expected": [ "32", "36", "0xe420264a00000000000000000000000000000000000000000000000000000000", "0x0000000600000000000000000000000000000000000000000000000000000000" ]
//! }, {
//!     "name": "abort_order",
//!     "inputs": [ { "method": "abortOrder", "calldata": [] } ],
//!     "expected": { "return_data": [ "0x4e487b7100000000000000000000000000000000000000000000000000000000", "0x0000002100000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    enum Choice { First, Second }

    string signature = "f(uint256)";

    function rename() internal returns (uint256) {
        signature = "g(uint256)";
        return 6;
    }

    function storageSignature() public returns (bytes memory) {
        return abi.encodeWithSignature(signature, rename());
    }

    function abortOrder() public returns (bytes memory) {
        Choice dirty;
        assembly {
            dirty := 2
            sstore(signature.slot, 1)
        }
        return abi.encodeWithSignature(signature, dirty);
    }
}
