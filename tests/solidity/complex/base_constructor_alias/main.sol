// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import {Base as Aliased, Marker} from "./base.sol";

contract Test is Aliased, Marker {
    constructor() Aliased(7) Marker() {}
}
