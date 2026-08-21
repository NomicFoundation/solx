// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./module.sol" as M;
import * as Star from "./module.sol";

contract Test {
    function plain() public pure returns (uint256) {
        return M.half(8);
    }

    function chained() public pure returns (uint256) {
        return M.Mod.half(8);
    }

    function starred() public pure returns (uint256) {
        return Star.half(8);
    }

    function chain() public pure returns (uint256) {
        return M.Nested.FREE_K;
    }
}
