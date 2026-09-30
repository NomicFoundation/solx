// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import {Node as OtherNode} from "./second.sol";

struct Node {
    Node[] kids;
    uint256 v;
}

contract First {
    Node node;
    OtherNode other;

    function run() public returns (uint256 first, uint256 second) {
        node.v = 7;
        other.v = 9;
        assembly {
            first := sload(1)
            second := sload(2)
        }
    }
}
