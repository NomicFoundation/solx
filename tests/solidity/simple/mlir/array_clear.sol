//! {
//!     "modes": [
//!         "Y"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "fill_st5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_st5_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "copy_st3_to_st5(uint256[3])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10",
//!                         "20",
//!                         "30"
//!                     ],
//!                     "expected": [
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_st5_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fill_dyn5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_dyn_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "shrink_dyn_to_3(uint256[3])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "100",
//!                         "200",
//!                         "300"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "3",
//!                         "100",
//!                         "200",
//!                         "300"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_dyn_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fill_dd3()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_dd_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "shrink_dd(uint256[][])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x20",
//!                         "1",
//!                         "10"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "1",
//!                         "0x20",
//!                         "1",
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_dd_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fill_sd3()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_sd3_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "copy_sd2_into_sd3(uint256[][2])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "0x40",
//!                         "0x80",
//!                         "1",
//!                         "10",
//!                         "1",
//!                         "20"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "0x60",
//!                         "0xa0",
//!                         "0xe0",
//!                         "1",
//!                         "10",
//!                         "1",
//!                         "20",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_sd3_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fill_ds3()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_ds_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "shrink_ds_to_1(uint256[3][])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "10",
//!                         "20",
//!                         "30"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "1",
//!                         "10",
//!                         "20",
//!                         "30"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_ds_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fill_sstr4()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_sstr4_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "copy_str2_to_sstr4(string[2])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000040",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7878000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7979000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                         "0x00000000000000000000000000000000000000000000000000000000000000c0",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000100",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000120",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7878000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7979000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_sstr4_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fill_sdyn_str4()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "check_sdyn_str_tail_filled()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "shrink_sdyn_str_to_2(string[2])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000040",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7878000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7979000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000040",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7878000000000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000002",
//!                         "0x7979000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "check_sdyn_str_tail_cleared()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

// Tests for storage array tail-clearing.
// Each test case pre-populates the destination with nonzero values, then
// copies a shorter source into it.  The vacated storage slots are verified
// with inline assembly sload so that the raw slot values are checked directly,
// independent of Solidity bounds checking or array length.  Cases:
//   1. Static dst (uint[5]) <- shorter static src (uint[3]): scalar tail cleared.
//   2. Dynamic dst (uint[]) shrinks from 5 to 3 elements: scalar tail cleared.
//   3. Dynamic dst (uint[][]) shrinks from 3 to 1 row: inner dynamic-array
//      length slots and their data areas must be zeroed.
//   4. Static dst (uint[][3]) <- shorter static src (uint[][2]): the
//      out-of-range inner dynamic array (slot [2]) must be cleared.
//   5. Dynamic dst (uint[3][]) shrinks from 3 to 1 row: static-inner array
//      storage slots (3 per row) must be zeroed.
//   6. Static dst (string[4]) <- shorter static src (string[2]): string tail
//      slots must be zeroed (short-string single-slot encoding).
//   7. Dynamic dst (string[]) shrinks from 4 to 2 elements: cleared string
//      slots at indices 2 and 3 must be zero.

contract Test {

    // Storage layout (slots allocated in declaration order):
    //   st5      : slots 0-4   (uint[5], 5 consecutive slots)
    //   dyn      : slot 5      (uint[], length slot; data at keccak256(5))
    //   dd       : slot 6      (uint[][], length slot; outer data at keccak256(6))
    //   sd3      : slots 7-9   (uint[][3]; sd3[i].length at slot 7+i)
    //   ds       : slot 10     (uint[3][], length slot; data at keccak256(10))
    //   sstr4    : slots 11-14 (string[4]; one slot per element, short-string encoding)
    //   sdyn_str : slot 15     (string[], length slot; data at keccak256(15))

    // uint[3] calldata -> uint[5] storage
    uint[5] st5;

    function fill_st5() public {
        st5 = [uint(1), 2, 3, 4, 5];
    }

    /// Verify the tail slots written by fill_st5 contain their expected values.
    function check_st5_tail_filled() external view returns (bool) {
        uint256 s3; uint256 s4;
        assembly {
            s3 := sload(add(st5.slot, 3))
            s4 := sload(add(st5.slot, 4))
        }
        return s3 == 4 && s4 == 5;
    }

    function copy_st3_to_st5(uint[3] calldata src) external returns (uint[5] memory) {
        st5 = src;
        return st5;
    }

    /// st5[3] (st5.slot+3) and st5[4] (st5.slot+4) must be zero.
    function check_st5_tail_cleared() external view returns (bool) {
        uint256 s3; uint256 s4;
        assembly {
            s3 := sload(add(st5.slot, 3))
            s4 := sload(add(st5.slot, 4))
        }
        return s3 == 0 && s4 == 0;
    }

    // uint[] shrinks from 5 to 3 elements
    uint[] dyn;

    function fill_dyn5() public {
        dyn = [uint(10), 20, 30, 40, 50];
    }

    /// Verify the tail slots written by fill_dyn5 contain their expected values.
    function check_dyn_tail_filled() external view returns (bool) {
        uint256 e3; uint256 e4;
        assembly {
            mstore(0x00, dyn.slot)
            let dataBase := keccak256(0x00, 0x20)
            e3 := sload(add(dataBase, 3))
            e4 := sload(add(dataBase, 4))
        }
        return e3 == 40 && e4 == 50;
    }

    function shrink_dyn_to_3(uint[3] calldata src) external returns (uint[] memory) {
        dyn = src;
        return dyn;
    }

    /// Data slots keccak256(dyn.slot)+3 and +4 must be zero after shrink.
    function check_dyn_tail_cleared() external view returns (bool) {
        uint256 e3; uint256 e4;
        assembly {
            mstore(0x00, dyn.slot)
            let dataBase := keccak256(0x00, 0x20)
            e3 := sload(add(dataBase, 3))
            e4 := sload(add(dataBase, 4))
        }
        return e3 == 0 && e4 == 0;
    }

    // uint[][] shrinks from 3 rows to 1 row
    uint[][] dd;

    function fill_dd3() public {
        dd.push();
        dd[0].push(1); dd[0].push(2); dd[0].push(3);
        dd.push();
        dd[1].push(4); dd[1].push(5); dd[1].push(6);
        dd.push();
        dd[2].push(7); dd[2].push(8); dd[2].push(9);
    }

    /// Verify the tail slots written by fill_dd3 contain their expected values.
    function check_dd_tail_filled() external view returns (bool) {
        uint256 d0e1; uint256 d0e2;
        uint256 len1; uint256 len2;
        uint256 d1e0; uint256 d1e1; uint256 d1e2;
        uint256 d2e0; uint256 d2e1; uint256 d2e2;
        assembly {
            mstore(0x00, dd.slot)
            let outerBase := keccak256(0x00, 0x20)
            // dd[0] tail: elements at indices 1 and 2 (values 2 and 3)
            mstore(0x00, outerBase)
            let dataBase0 := keccak256(0x00, 0x20)
            d0e1 := sload(add(dataBase0, 1))
            d0e2 := sload(add(dataBase0, 2))
            let lenSlot1 := add(outerBase, 1)
            let lenSlot2 := add(outerBase, 2)
            len1 := sload(lenSlot1)
            len2 := sload(lenSlot2)
            mstore(0x00, lenSlot1)
            let dataBase1 := keccak256(0x00, 0x20)
            d1e0 := sload(dataBase1)
            d1e1 := sload(add(dataBase1, 1))
            d1e2 := sload(add(dataBase1, 2))
            mstore(0x00, lenSlot2)
            let dataBase2 := keccak256(0x00, 0x20)
            d2e0 := sload(dataBase2)
            d2e1 := sload(add(dataBase2, 1))
            d2e2 := sload(add(dataBase2, 2))
        }
        return d0e1 == 2 && d0e2 == 3
            && len1 == 3 && len2 == 3
            && d1e0 == 4 && d1e1 == 5 && d1e2 == 6
            && d2e0 == 7 && d2e1 == 8 && d2e2 == 9;
    }

    function shrink_dd(uint[][] calldata src) external returns (uint[][] memory) {
        dd = src;
        return dd;
    }

    /// After shrink to 1 row with 1 element:
    ///   dd[0] tail: keccak256(outerBase)+{1,2} must be 0
    ///   length slots  keccak256(dd.slot)+1  and +2  must be 0
    ///   data areas    keccak256(len_slot_i)+{0,1,2} must be 0
    function check_dd_tail_cleared() external view returns (bool) {
        uint256 d0e1; uint256 d0e2;
        uint256 len1; uint256 len2;
        uint256 d1e0; uint256 d1e1; uint256 d1e2;
        uint256 d2e0; uint256 d2e1; uint256 d2e2;
        assembly {
            mstore(0x00, dd.slot)
            let outerBase := keccak256(0x00, 0x20)
            // dd[0] tail
            mstore(0x00, outerBase)
            let dataBase0 := keccak256(0x00, 0x20)
            d0e1 := sload(add(dataBase0, 1))
            d0e2 := sload(add(dataBase0, 2))
            let lenSlot1 := add(outerBase, 1)
            let lenSlot2 := add(outerBase, 2)
            len1 := sload(lenSlot1)
            len2 := sload(lenSlot2)
            mstore(0x00, lenSlot1)
            let dataBase1 := keccak256(0x00, 0x20)
            d1e0 := sload(dataBase1)
            d1e1 := sload(add(dataBase1, 1))
            d1e2 := sload(add(dataBase1, 2))
            mstore(0x00, lenSlot2)
            let dataBase2 := keccak256(0x00, 0x20)
            d2e0 := sload(dataBase2)
            d2e1 := sload(add(dataBase2, 1))
            d2e2 := sload(add(dataBase2, 2))
        }
        return d0e1 == 0 && d0e2 == 0
            && len1 == 0 && len2 == 0
            && d1e0 == 0 && d1e1 == 0 && d1e2 == 0
            && d2e0 == 0 && d2e1 == 0 && d2e2 == 0;
    }

    // uint[][2] calldata -> uint[][3] storage
    uint[][3] sd3;

    function fill_sd3() public {
        sd3[0].push(1); sd3[0].push(2);
        sd3[1].push(3); sd3[1].push(4);
        sd3[2].push(5); sd3[2].push(6);
    }

    /// Verify the tail slots written by fill_sd3 contain their expected values.
    function check_sd3_tail_filled() external view returns (bool) {
        uint256 d0e1; uint256 d1e1;
        uint256 len2; uint256 d2e0; uint256 d2e1;
        assembly {
            // sd3[0][1] == 2 (tail of row 0, shrinks from 2 to 1)
            mstore(0x00, sd3.slot)
            let dataBase0 := keccak256(0x00, 0x20)
            d0e1 := sload(add(dataBase0, 1))
            // sd3[1][1] == 4 (tail of row 1, shrinks from 2 to 1)
            mstore(0x00, add(sd3.slot, 1))
            let dataBase1 := keccak256(0x00, 0x20)
            d1e1 := sload(add(dataBase1, 1))
            // sd3[2]: entirely cleared (length 2, elements 5 and 6)
            let lenSlot2 := add(sd3.slot, 2)
            len2 := sload(lenSlot2)
            mstore(0x00, lenSlot2)
            let dataBase2 := keccak256(0x00, 0x20)
            d2e0 := sload(dataBase2)
            d2e1 := sload(add(dataBase2, 1))
        }
        return d0e1 == 2 && d1e1 == 4
            && len2 == 2 && d2e0 == 5 && d2e1 == 6;
    }

    function copy_sd2_into_sd3(uint[][2] calldata src) external returns (uint[][3] memory) {
        sd3 = src;
        return sd3;
    }

    /// sd3[0][1] and sd3[1][1] must be 0 (row tails shrunk from 2 to 1).
    /// sd3[2].length (sd3.slot+2) must be 0.
    /// sd3[2]'s data area keccak256(sd3.slot+2)+{0,1} must be 0 (old length was 2).
    function check_sd3_tail_cleared() external view returns (bool) {
        uint256 d0e1; uint256 d1e1;
        uint256 len2; uint256 d2e0; uint256 d2e1;
        assembly {
            mstore(0x00, sd3.slot)
            let dataBase0 := keccak256(0x00, 0x20)
            d0e1 := sload(add(dataBase0, 1))
            mstore(0x00, add(sd3.slot, 1))
            let dataBase1 := keccak256(0x00, 0x20)
            d1e1 := sload(add(dataBase1, 1))
            let lenSlot2 := add(sd3.slot, 2)
            len2 := sload(lenSlot2)
            mstore(0x00, lenSlot2)
            let dataBase2 := keccak256(0x00, 0x20)
            d2e0 := sload(dataBase2)
            d2e1 := sload(add(dataBase2, 1))
        }
        return d0e1 == 0 && d1e1 == 0
            && len2 == 0 && d2e0 == 0 && d2e1 == 0;
    }

    // uint[3][] shrinks from 3 rows to 1 row
    uint[3][] ds;

    function fill_ds3() public {
        ds.push();
        ds[0] = [uint(1), 2, 3];
        ds.push();
        ds[1] = [uint(4), 5, 6];
        ds.push();
        ds[2] = [uint(7), 8, 9];
    }

    /// Verify the tail slots written by fill_ds3 contain their expected values.
    function check_ds_tail_filled() external view returns (bool) {
        uint256 r1e0; uint256 r1e1; uint256 r1e2;
        uint256 r2e0; uint256 r2e1; uint256 r2e2;
        assembly {
            mstore(0x00, ds.slot)
            let base := keccak256(0x00, 0x20)
            r1e0 := sload(add(base, 3))
            r1e1 := sload(add(base, 4))
            r1e2 := sload(add(base, 5))
            r2e0 := sload(add(base, 6))
            r2e1 := sload(add(base, 7))
            r2e2 := sload(add(base, 8))
        }
        return r1e0 == 4 && r1e1 == 5 && r1e2 == 6
            && r2e0 == 7 && r2e1 == 8 && r2e2 == 9;
    }

    function shrink_ds_to_1(uint[3][] calldata src) external returns (uint[3][] memory) {
        ds = src;
        return ds;
    }

    /// After shrink to 1 row, the 6 data slots for rows 1 and 2 must be 0:
    ///   row 1: keccak256(ds.slot)+{3,4,5}
    ///   row 2: keccak256(ds.slot)+{6,7,8}
    function check_ds_tail_cleared() external view returns (bool) {
        uint256 r1e0; uint256 r1e1; uint256 r1e2;
        uint256 r2e0; uint256 r2e1; uint256 r2e2;
        assembly {
            mstore(0x00, ds.slot)
            let base := keccak256(0x00, 0x20)
            r1e0 := sload(add(base, 3))
            r1e1 := sload(add(base, 4))
            r1e2 := sload(add(base, 5))
            r2e0 := sload(add(base, 6))
            r2e1 := sload(add(base, 7))
            r2e2 := sload(add(base, 8))
        }
        return r1e0 == 0 && r1e1 == 0 && r1e2 == 0
            && r2e0 == 0 && r2e1 == 0 && r2e2 == 0;
    }

    // string[2] calldata -> string[4] storage
    string[4] sstr4;

    function fill_sstr4() public {
        sstr4[0] = "aa"; sstr4[1] = "bb"; sstr4[2] = "cc"; sstr4[3] = "dd";
    }

    /// Verify that slots 2 and 3 of sstr4 are non-zero after fill.
    /// Short strings are packed into one slot: data (high bytes) + length*2 (low byte).
    function check_sstr4_tail_filled() external view returns (bool) {
        uint256 s2; uint256 s3;
        assembly {
            s2 := sload(add(sstr4.slot, 2))
            s3 := sload(add(sstr4.slot, 3))
        }
        return s2 != 0 && s3 != 0;
    }

    function copy_str2_to_sstr4(string[2] calldata src) external returns (string[4] memory) {
        sstr4 = src;
        string[4] memory r;
        r[0] = sstr4[0]; r[1] = sstr4[1]; r[2] = sstr4[2]; r[3] = sstr4[3];
        return r;
    }

    /// sstr4[2] (sstr4.slot+2) and sstr4[3] (sstr4.slot+3) must be zero.
    function check_sstr4_tail_cleared() external view returns (bool) {
        uint256 s2; uint256 s3;
        assembly {
            s2 := sload(add(sstr4.slot, 2))
            s3 := sload(add(sstr4.slot, 3))
        }
        return s2 == 0 && s3 == 0;
    }

    // string[] shrinks from 4 to 2 elements
    string[] sdyn_str;

    function fill_sdyn_str4() public {
        sdyn_str.push("aa"); sdyn_str.push("bb");
        sdyn_str.push("cc"); sdyn_str.push("dd");
    }

    /// Verify that data slots at indices 2 and 3 are non-zero after fill.
    function check_sdyn_str_tail_filled() external view returns (bool) {
        uint256 e2; uint256 e3;
        assembly {
            mstore(0x00, sdyn_str.slot)
            let dataBase := keccak256(0x00, 0x20)
            e2 := sload(add(dataBase, 2))
            e3 := sload(add(dataBase, 3))
        }
        return e2 != 0 && e3 != 0;
    }

    function shrink_sdyn_str_to_2(string[2] calldata src) external returns (string[] memory) {
        sdyn_str = src;
        uint len = sdyn_str.length;
        string[] memory r = new string[](len);
        for (uint i = 0; i < len; i++) r[i] = sdyn_str[i];
        return r;
    }

    /// Data slots keccak256(sdyn_str.slot)+2 and +3 must be zero after shrink.
    function check_sdyn_str_tail_cleared() external view returns (bool) {
        uint256 e2; uint256 e3;
        assembly {
            mstore(0x00, sdyn_str.slot)
            let dataBase := keccak256(0x00, 0x20)
            e2 := sload(add(dataBase, 2))
            e3 := sload(add(dataBase, 3))
        }
        return e2 == 0 && e3 == 0;
    }
}
