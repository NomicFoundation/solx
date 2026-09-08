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
        arr4[0].a = 255;
        arr4[0].c72[0] = type(uint72).max;
        arr4[0].c72[1] = type(uint72).max;
        bytes19 tmpB = hex"deadbeef";
        arr4[0].b = tmpB;
        bytes17 tmpC = hex"cafebabe";
        arr4[0].c[0] = tmpC;
        arr4[0].c[1] = tmpC;
        arr4[0].c[2] = tmpC;
        arr4[0].f = type(uint256).max;
        arr4[0].flag = true;
        arr4[0].direction = Direction.West;
        arr4[0].fixedAddrArr[0] = address(0xdead);
        arr4[0].fixedAddrArr[1] = address(0xbeef);
        // Seed fn_uint slot (14) with non-zero data via assembly.
        assembly {
            mstore(0, arr4.slot)
            let base := keccak256(0, 0x20)
            sstore(add(base, 14), 0xffffffffffffffffffffffffffffffffffffffffffffffff)
        }

        arr4.pop();

        assembly {
            mstore(0, arr4.slot)
            let base := keccak256(0, 0x20)
            if sload(base)           { revert(0, 0) }   // slot  0: a
            if sload(add(base,  1))  { revert(0, 0) }   // slot  1: c72[0..1]
            if sload(add(base,  2))  { revert(0, 0) }   // slot  2: b
            if sload(add(base,  3))  { revert(0, 0) }   // slot  3: c[0]
            if sload(add(base,  4))  { revert(0, 0) }   // slot  4: c[1]
            if sload(add(base,  5))  { revert(0, 0) }   // slot  5: c[2]
            if sload(add(base,  8))  { revert(0, 0) }   // slot  8: f
            if sload(add(base,  9))  { revert(0, 0) }   // slot  9: flag+direction
            if sload(add(base, 12))  { revert(0, 0) }   // slot 12: fixedAddrArr[0]
            if sload(add(base, 13))  { revert(0, 0) }   // slot 13: fixedAddrArr[1]
            if sload(add(base, 14))  { revert(0, 0) }   // slot 14: fn_uint
        }
        assert(arr4.length == 0);
    }
}
