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
//!         "5"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The cycle runs through a member laid out inline: `Outer` holds an `Inner`, which holds an array
// of `Outer`; `Framed` holds a fixed-size array of `Boxed`, which holds an array of `Framed`; `A`
// holds a `B` holding a `C`, which holds an array of `B`. The trailing member sits after a whole
// inner struct, so writing it must not land on an array's slot. `first`, `third` and `fifth` are
// declared before the structs that hold them, so type resolution enters at the array side and
// meets the inner struct while it is still being built.

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
            uint256 chainTail
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
        sixth.tail = 5;
        chainLength = sixth.b.c.bs.length;
        chainTail = sixth.tail;
    }
}
