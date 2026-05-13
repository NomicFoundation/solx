//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "testCheckEntrypointDoesNotHitInvalidBytecode",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract PlaceholderContract {
    function check_entrypoint(
        bool, bool, bool, bool, bool,
        bool, bool, bool, bool, bool,
        bool, bool, bool, bool, bool
    ) public pure returns (bool) {
        return true;
    }
}

contract Test {
    function testCheckEntrypointDoesNotHitInvalidBytecode() public returns (bool) {
        PlaceholderContract target = new PlaceholderContract();
        return target.check_entrypoint(
            false, false, false, false, false,
            false, false, false, false, false,
            false, false, false, false, false
        );
    }
}
