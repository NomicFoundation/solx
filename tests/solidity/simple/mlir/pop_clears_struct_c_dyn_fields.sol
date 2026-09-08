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
        // d: out-of-place string (>31 bytes triggers multi-slot storage).
        arr4[0].d = "a long string value that exceeds thirty-one bytes easily";
        // e: three elements so that genClearStorageArrayTail loops 3 times.
        arr4[0].e.push(111);
        arr4[0].e.push(222);
        arr4[0].e.push(333);

        arr4.pop();

        assembly {
            mstore(0, arr4.slot)
            let base := keccak256(0, 0x20)

            // slot 6: d length/encoding word must be zero.
            let dSlot := add(base, 6)
            if sload(dSlot) { revert(0, 0) }
            // First data chunk of d must be zero.
            mstore(0, dSlot)
            if sload(keccak256(0, 0x20)) { revert(0, 0) }

            // slot 7: e length must be zero.
            let eSlot := add(base, 7)
            if sload(eSlot) { revert(0, 0) }
            // e data slots 0-2 must be zero.
            mstore(0, eSlot)
            let eData := keccak256(0, 0x20)
            if sload(eData)           { revert(0, 0) }
            if sload(add(eData, 1))   { revert(0, 0) }
            if sload(add(eData, 2))   { revert(0, 0) }
        }
        assert(arr4.length == 0);
    }
}
