// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import {Base as Aliased} from "./base.sol";

contract Test is Aliased {
    constructor() Aliased(7) {}
}
