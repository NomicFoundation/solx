// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0 || 0.8.30;
pragma experimental ABIEncoderV2;

import {IERC165 as Introspection, ICounter} from "Base.sol";

// Größe: the multi-byte characters above shift every later byte offset past its character offset.

type Price is uint128;

uint256 constant LIMIT = 10;

struct Point {
    uint256 x;
    uint256 y;
}

enum Side {
    Buy,
    Sell
}

event Moved(uint256 indexed x);

error TooFar(uint256 by);

function half(uint256 x) pure returns (uint256) {
    return x / 2;
}

using {half} for uint256;

function add(Price a, Price b) pure returns (Price) {
    return Price.wrap(Price.unwrap(a) + Price.unwrap(b));
}

function sub(Price a, Price b) pure returns (Price) {
    return Price.wrap(Price.unwrap(a) - Price.unwrap(b));
}

using {add as +, sub as -} for Price global;

interface IKinds {
    function statements(uint256 n) external returns (uint256 total);

    function yul(uint256 n) external pure returns (uint256 result);
}

contract Callee {
    constructor(uint256 seed) payable {}

    function pay(uint256 a, uint256 b) external payable returns (uint256, uint256) {
        return (b, a);
    }
}

abstract contract Child is Callee(1), IKinds {}

contract Kinds {
    function(uint256) internal pure returns (uint256) private scale = half;
    address payable private sink;

    modifier atMost(uint256 n, uint256 cap) {
        require(n <= cap);
        _;
    }

    modifier positive(uint256 n) {
        require(n > 0);
        _;
    }

    function statements(uint256 n) external atMost(n, 50) positive(n) returns (uint256 total) {
        for (uint256 i = 0; i < n; i++) {
            if (i == 3) {
                total += 1;
                continue;
            } else if (i > LIMIT) {
                total += 2;
                break;
            }
            total += i;
        }
        while (total > 100) {
            total -= 1;
        }
        do {
            total = total << 1;
        } while (total == 0);
        unchecked {
            total = total ** 2;
        }
        if (total == 7) {
            total += 3;
            revert TooFar(total);
        }
        emit Moved(total);
    }

    function expressions(bool flag) external returns (uint256) {
        (uint256 a, uint256 b) = (1, 2);
        (a, b) = (b, a);
        uint256[] memory values = new uint256[](3);
        uint8[2] memory pair = [uint8(1), 2];
        values[0] = flag && !flag || flag ? a | b & a ^ 0xff : -int256(a) > 0 ? 1 : 0;
        Callee callee = new Callee(1);
        try callee.pay{value: 0, gas: 50000}({a: a, b: pair[1]}) returns (uint256 x, uint256) {
            a = x.half() + scale(type(uint256).max % 7) * 3 - 1;
        } catch Error(string memory) {
            a = 0;
        } catch {
            a = 1;
        }
        bytes memory text = bytes(string.concat("a" "b", unicode"é"));
        bytes memory raw = hex"00" hex"01";
        return a + text.length + raw.length + uint256(Price.unwrap(Price.wrap(2) - Price.wrap(1))) + uint256(Side.Sell);
    }

    function yul(uint256 n) external pure returns (uint256 result) {
        assembly {
            function twice(v) -> w {
                w := add(v, v)
                leave
            }
            function start() -> a, b {
                b := 1
            }
            let i, j := start()
            i, j := start()
            for { } lt(i, n) { i := add(i, 1) } {
                if eq(i, 5) {
                    j := 0
                    break
                }
                if eq(i, 2) {
                    j := 1
                    continue
                }
                j := twice(j)
            }
            switch j
            case 0 { result := 1 }
            case 1 { result := 2 }
            default { result := j }
        }
        result += 1;
    }
}
