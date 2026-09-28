//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "delNested()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x3fffffffffffffffffffffffffffff00000000",
//!                         "0x3fffffffffffffffffffffffffffff00000000",
//!                         "0x3fffffffffffffffffffffffffffff00000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delPair()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x3fffff00000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delWithArrays()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x3fffffffffffffffffffffffffffff00000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delFull()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delStaticArrayOfStructs()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x3fffffffffffffffffffffffffffff00000000",
//!                         "0x3fffffffffffffffffffffffffffff00000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "delNestedStaticArrayOfStructs()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x3fffffffffffffffffffffffffffff00000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    struct Nested { uint32 a; Nested[] x; }
    struct Pair { uint64 y; uint64 z; }
    struct WithArrays { uint32 a; uint32[3] b; uint32[] x; }
    struct Full { uint256 a; }
    struct Packed { uint32 a; }

    Nested nested;
    Pair pair;
    WithArrays withArrays;
    Full full;
    Packed[5] five;
    Packed[2][2] grid;

    // One member, 4 of 32 bytes: 28 bytes of padding survive, in the struct's
    // own slot and in both cleared elements of the nested array.
    function delNested() public returns (uint256 r1, uint256 r2, uint256 r3) {
        assembly {
            // 2 ** 150 - 1
            sstore(nested.slot, 1427247692705959881058285969449495136382746623)
        }
        nested.a = 1;
        nested.x.push(); nested.x.push();
        Nested storage p1 = nested.x[0];
        Nested storage p2 = nested.x[1];
        assembly {
            // 2 ** 150 - 1
            sstore(p1.slot, 1427247692705959881058285969449495136382746623)
            sstore(p2.slot, 1427247692705959881058285969449495136382746623)
        }
        nested.x[0].a = 2; nested.x[1].a = 3;
        delete nested;
        assert(nested.a == 0);
        assert(nested.x.length == 0);
        assembly {
            r1 := sload(nested.slot)
            r2 := sload(p1.slot)
            r3 := sload(p2.slot)
        }
    }

    // Two members packed into one slot: 16 of 32 bytes cleared, 16 survive.
    function delPair() public returns (uint256 r) {
        assembly {
            // 2 ** 150 - 1
            sstore(pair.slot, 1427247692705959881058285969449495136382746623)
        }
        pair.y = 1; pair.z = 2;
        delete pair;
        assert(pair.y == 0);
        assert(pair.z == 0);
        assembly { r := sload(pair.slot) }
    }

    // A value member followed by array members, which are slot aligned and so
    // never share the first slot: the same 28 bytes of padding survive.
    function delWithArrays() public returns (uint256 r) {
        assembly {
            // 2 ** 150 - 1
            sstore(withArrays.slot, 1427247692705959881058285969449495136382746623)
        }
        withArrays.a = 1;
        withArrays.b[0] = 2; withArrays.b[1] = 3;
        withArrays.x.push(4); withArrays.x.push(5);
        delete withArrays;
        assert(withArrays.a == 0);
        assert(withArrays.b[0] == 0);
        assert(withArrays.b[1] == 0);
        assert(withArrays.x.length == 0);
        assembly { r := sload(withArrays.slot) }
    }

    // A member that fills its slot leaves no padding, so the slot reads zero.
    function delFull() public returns (uint256 r) {
        assembly {
            // 2 ** 150 - 1
            sstore(full.slot, 1427247692705959881058285969449495136382746623)
        }
        full.a = 1;
        delete full;
        assembly { r := sload(full.slot) }
    }

    // A static array of packed structs, longer than the four elements the old
    // code generator unrolls, so its clearing loop runs: every element keeps
    // its padding, not only the first.
    function delStaticArrayOfStructs() public returns (uint256 r0, uint256 r4) {
        Packed storage p0 = five[0];
        Packed storage p4 = five[4];
        assembly {
            // 2 ** 150 - 1
            sstore(p0.slot, 1427247692705959881058285969449495136382746623)
            sstore(p4.slot, 1427247692705959881058285969449495136382746623)
        }
        five[0].a = 1; five[4].a = 5;
        delete five;
        assert(five[0].a == 0);
        assert(five[4].a == 0);
        assembly {
            r0 := sload(p0.slot)
            r4 := sload(p4.slot)
        }
    }

    // A nested static array of packed structs: the outer array's elements are
    // arrays, and the padding survives through both levels.
    function delNestedStaticArrayOfStructs() public returns (uint256 r) {
        Packed storage p = grid[1][1];
        assembly {
            // 2 ** 150 - 1
            sstore(p.slot, 1427247692705959881058285969449495136382746623)
        }
        grid[1][1].a = 1;
        delete grid;
        assert(grid[1][1].a == 0);
        assembly { r := sload(p.slot) }
    }
}
