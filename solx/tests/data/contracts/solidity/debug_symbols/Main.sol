// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "Base.sol";

function double(uint256 x) pure returns (uint256) {
    return x * 2;
}

library MathLib {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }

    function triple(uint256 x) public pure returns (uint256) {
        return x * 3;
    }
}

abstract contract Left is Root {
    function hook(uint256 x) internal virtual override returns (uint256) {
        return x + 1;
    }
}

abstract contract Right is Root {
    function hook(uint256 x) internal virtual override returns (uint256) {
        return x + 2;
    }
}

contract Counter is Left, Right {
    address private owner;
    uint256 public override count;
    mapping(address => uint256) public balances;
    uint256[] public history;

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }

    constructor() payable {
        owner = msg.sender;
    }

    function inc() public onlyOwner {
        count = MathLib.add(count, 1);
        history.push(count);
    }

    function inc(uint256 by) external onlyOwner {
        count = double(by) + hook(count);
    }

    function hook(uint256 x) internal override(Left, Right) returns (uint256) {
        return super.hook(x);
    }

    function reset() private {
        count = 0;
    }

    fallback() external payable {}

    receive() external payable {}
}

pragma abicoder v2;
