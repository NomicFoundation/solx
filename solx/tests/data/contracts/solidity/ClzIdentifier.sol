// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract ClzIdentifier {
    function f() public pure returns (uint256 r) {
        assembly {
            let clz := 1
            r := clz
        }
    }
}
