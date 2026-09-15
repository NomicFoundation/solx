//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "test()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "#storage_empty",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": []
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

enum Direction { North, South, East, West }
struct Inner { uint256 x; bool[][] flags; }
struct C {
    uint8 a;
    uint72[2] c72;
    bytes19 b;
    bytes17[3] c;
    string d;
    uint256[] e;
    uint256 f;
    bool flag;
    Direction direction;
    Inner inner;
    address[2] fixedAddrArr;
    function(uint256) external fn_uint;
}
contract Test {
    C[] arr4;

    function test() public {
        arr4.push();
        arr4[0].inner.x = 0xdeadbeef;

        // flags[0]: two bools (packed into one data slot)
        arr4[0].inner.flags.push();
        arr4[0].inner.flags[0].push(true);
        arr4[0].inner.flags[0].push(false);

        // flags[1]: one bool
        arr4[0].inner.flags.push();
        arr4[0].inner.flags[1].push(true);

        arr4.pop();

        assembly {
            mstore(0, arr4.slot)
            let base := keccak256(0, 0x20)

            // slot 10: inner.x must be zero.
            if sload(add(base, 10)) { revert(0, 0) }

            // slot 11: flags length (outer bool[][]) must be zero.
            let flagsSlot := add(base, 11)
            if sload(flagsSlot) { revert(0, 0) }

            // flags data starts at keccak256(flagsSlot).
            // flags[i].length lives at flagsData + i.
            mstore(0, flagsSlot)
            let flagsData := keccak256(0, 0x20)
            if sload(flagsData)         { revert(0, 0) }  // flags[0].length
            if sload(add(flagsData, 1)) { revert(0, 0) }  // flags[1].length

            // flags[0] element data at keccak256(flagsData + 0).
            mstore(0, flagsData)
            if sload(keccak256(0, 0x20)) { revert(0, 0) }

            // flags[1] element data at keccak256(flagsData + 1).
            mstore(0, add(flagsData, 1))
            if sload(keccak256(0, 0x20)) { revert(0, 0) }
        }
        assert(arr4.length == 0);
    }
}
