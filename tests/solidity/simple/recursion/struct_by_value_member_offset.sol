//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "run",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "2",
//!         "7",
//!         "1",
//!         "9",
//!         "3",
//!         "5",
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// `first`, `third` and `fifth` are declared before their holders, so resolution enters each cycle
// at the array side.

pragma solidity >=0.8.0;

contract Test {
    struct Outer {
        Inner inner;
        uint256 tail;
    }

    struct Inner {
        Outer[] items;
    }

    struct Framed {
        Boxed[2] boxes;
        uint256 tail;
    }

    struct Boxed {
        Framed[] items;
    }

    struct B {
        A[] items;
        C c;
    }

    struct A {
        B b;
        uint256 tail;
    }

    struct C {
        B[] bs;
    }

    Inner first;
    Outer second;
    Boxed third;
    Framed fourth;
    B fifth;
    A sixth;

    function run()
        public
        returns (
            uint256 length,
            uint256 tail,
            uint256 framedLength,
            uint256 framedTail,
            uint256 chainLength,
            uint256 chainTail,
            uint256 chainItems
        )
    {
        second.inner.items.push();
        second.inner.items.push();
        second.tail = 7;
        length = second.inner.items.length;
        tail = second.tail;

        fourth.boxes[0].items.push();
        fourth.tail = 9;
        framedLength = fourth.boxes[0].items.length;
        framedTail = fourth.tail;

        sixth.b.c.bs.push();
        sixth.b.c.bs.push();
        sixth.b.c.bs.push();
        sixth.b.items.push();
        sixth.tail = 5;
        chainLength = sixth.b.c.bs.length;
        chainTail = sixth.tail;
        chainItems = sixth.b.items.length;
    }
}
