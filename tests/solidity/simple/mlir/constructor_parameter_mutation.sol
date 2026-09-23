//! {
//!     "modes": ["E"],
//!     "cases": [{
//!         "name": "default",
//!         "inputs": [
//!             {
//!                 "method": "forward()",
//!                 "calldata": [],
//!                 "expected": ["8", "8", "8"]
//!             },
//!             {
//!                 "method": "reverse()",
//!                 "calldata": [],
//!                 "expected": ["8", "7", "8"]
//!             },
//!             {
//!                 "method": "provider()",
//!                 "calldata": [],
//!                 "expected": ["8", "7", "8", "7"]
//!             },
//!             {
//!                 "method": "referenceParameters()",
//!                 "calldata": [],
//!                 "expected": ["7", "8", "8"]
//!             }
//!         ]
//!     }]
//! }

// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0;

abstract contract Root {
    uint256 public root;

    constructor(uint256 seed) {
        root = seed;
    }
}

abstract contract Middle is Root {
    uint256 public middle;

    constructor(uint256 seed) {
        middle = seed;
    }
}

contract Forward is Middle {
    uint256 public derived;

    constructor(uint256 seed) Root(seed) Middle(++seed) {
        derived = seed;
    }
}

contract Reverse is Middle {
    uint256 public derived;

    constructor(uint256 seed) Root(++seed) Middle(seed) {
        derived = seed;
    }
}

abstract contract Provider is Middle {
    uint256 public provided;

    constructor(uint256 seed) Root(++seed) Middle(seed) {
        provided = seed;
    }
}

contract Provided is Provider {
    uint256 public derived;

    constructor(uint256 seed) Provider(seed) {
        derived = seed;
    }
}

abstract contract ReferenceRoot {
    uint256 public root;

    constructor(uint256[] memory seed) {
        root = seed[0];
        ++seed[0];
    }
}

abstract contract ReferenceMiddle is ReferenceRoot {
    uint256 public middle;

    constructor(uint256[] memory seed) {
        middle = seed[0];
    }
}

contract ReferenceLeaf is ReferenceMiddle {
    uint256 public derived;

    constructor(uint256[] memory seed) ReferenceRoot(seed) ReferenceMiddle(seed) {
        derived = seed[0];
    }
}

contract Test {
    function forward() public returns (uint256, uint256, uint256) {
        Forward instance = new Forward(7);
        return (instance.root(), instance.middle(), instance.derived());
    }

    function reverse() public returns (uint256, uint256, uint256) {
        Reverse instance = new Reverse(7);
        return (instance.root(), instance.middle(), instance.derived());
    }

    function provider() public returns (uint256, uint256, uint256, uint256) {
        Provided instance = new Provided(7);
        return (instance.root(), instance.middle(), instance.provided(), instance.derived());
    }

    function referenceParameters() public returns (uint256, uint256, uint256) {
        uint256[] memory seed = new uint256[](1);
        seed[0] = 7;
        ReferenceLeaf instance = new ReferenceLeaf(seed);
        return (instance.root(), instance.middle(), instance.derived());
    }
}
