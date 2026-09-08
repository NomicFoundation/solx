//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "run",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "7"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// Each struct names the next one twice, so a type walk that does not remember the structs it has
// entered visits the last one once per path through the chain: 2^30 times. The MLIR printer spells
// a literal struct's body at every occurrence and so takes as long, which keeps this a tester case.

pragma solidity >=0.8.0;

contract Test {
    struct S0 {
        S1[] items;
        S1 next;
    }

    struct S1 {
        S2[] items;
        S2 next;
    }

    struct S2 {
        S3[] items;
        S3 next;
    }

    struct S3 {
        S4[] items;
        S4 next;
    }

    struct S4 {
        S5[] items;
        S5 next;
    }

    struct S5 {
        S6[] items;
        S6 next;
    }

    struct S6 {
        S7[] items;
        S7 next;
    }

    struct S7 {
        S8[] items;
        S8 next;
    }

    struct S8 {
        S9[] items;
        S9 next;
    }

    struct S9 {
        S10[] items;
        S10 next;
    }

    struct S10 {
        S11[] items;
        S11 next;
    }

    struct S11 {
        S12[] items;
        S12 next;
    }

    struct S12 {
        S13[] items;
        S13 next;
    }

    struct S13 {
        S14[] items;
        S14 next;
    }

    struct S14 {
        S15[] items;
        S15 next;
    }

    struct S15 {
        S16[] items;
        S16 next;
    }

    struct S16 {
        S17[] items;
        S17 next;
    }

    struct S17 {
        S18[] items;
        S18 next;
    }

    struct S18 {
        S19[] items;
        S19 next;
    }

    struct S19 {
        S20[] items;
        S20 next;
    }

    struct S20 {
        S21[] items;
        S21 next;
    }

    struct S21 {
        S22[] items;
        S22 next;
    }

    struct S22 {
        S23[] items;
        S23 next;
    }

    struct S23 {
        S24[] items;
        S24 next;
    }

    struct S24 {
        S25[] items;
        S25 next;
    }

    struct S25 {
        S26[] items;
        S26 next;
    }

    struct S26 {
        S27[] items;
        S27 next;
    }

    struct S27 {
        S28[] items;
        S28 next;
    }

    struct S28 {
        S29[] items;
        S29 next;
    }

    struct S29 {
        S30[] items;
        S30 next;
    }

    struct S30 {
        uint256 tail;
    }

    S0 chain;

    function run() public returns (uint256 stored) {
        chain.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.tail = 7;
        assembly {
            stored := sload(30)
        }
    }
}
