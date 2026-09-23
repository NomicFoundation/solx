//! {
//!     "modes": ["E"],
//!     "cases": [{
//!         "name": "default",
//!         "inputs": [
//!             {
//!                 "method": "diamond()",
//!                 "calldata": [],
//!                 "value": "1 wei",
//!                 "expected": ["12345", "11", "1"]
//!             },
//!             {
//!                 "method": "synthesized()",
//!                 "calldata": [],
//!                 "expected": ["1234"]
//!             }
//!         ]
//!     }]
//! }

// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0;

abstract contract Root {
    uint256 public order = 1;

    constructor(uint256 seed) payable {
        order = order * 10 + 2;
        if (seed == 7) return;
        order = 99;
    }
}

abstract contract Left is Root {
    constructor() {
        order = order * 10 + 3;
    }
}

abstract contract Gap is Root(7), Left {}

abstract contract Right is Root {
    constructor() {
        order = order * 10 + 4;
    }
}

contract Diamond is Gap, Right {
    uint256 public initialized = order + 10;

    constructor() payable {
        order = order * 10 + 5;
    }
}

contract Synthesized is Gap, Right {}

contract Test {
    function diamond() public payable returns (uint256, uint256, uint256) {
        Diamond instance = new Diamond{value: msg.value}();
        return (instance.order(), instance.initialized(), address(instance).balance);
    }

    function synthesized() public returns (uint256) {
        return (new Synthesized()).order();
    }
}
