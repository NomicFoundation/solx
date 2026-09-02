// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "virt/Dep.sol" as D;

contract Main {
    function main() public pure returns (uint256) {
        return D.add(40, 2);
    }
}
