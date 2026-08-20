// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./library.sol" as M;

contract Test {
    function externalCall() public pure returns (uint256) {
        return M.Halver.half(8);
    }

    function tried() public returns (uint256) {
        try M.Halver.half(8) returns (uint256 value) {
            return value;
        } catch {
            return 0;
        }
    }
}
