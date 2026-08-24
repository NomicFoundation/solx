//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "sort(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "5",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "5",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "5",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sortReverse(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "5",
//!                         "3",
//!                         "1",
//!                         "2",
//!                         "5",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "5",
//!                         "5",
//!                         "4",
//!                         "3",
//!                         "2",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "misc(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "100",
//!                         "7"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "1",
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function _mload(uint256 p) private pure returns (uint256 v) {
    assembly ("memory-safe") {
      v := mload(p)
    }
  }

  function _swap(uint256 a, uint256 b) private pure {
    assembly ("memory-safe") {
      let t := mload(a)
      mstore(a, mload(b))
      mstore(b, t)
    }
  }

  function _quickSort(uint256 begin, uint256 end, function(uint256, uint256) pure returns (bool) comp) private pure {
    unchecked {
      if (end - begin < 0x40) return;
      uint256 pivot = _mload(begin);
      uint256 pos = begin;
      for (uint256 it = begin + 0x20; it < end; it += 0x20) {
        if (comp(_mload(it), pivot)) {
          pos += 0x20;
          _swap(pos, it);
        }
      }
      _swap(begin, pos);
      _quickSort(begin, pos, comp);
      _quickSort(pos + 0x20, end, comp);
    }
  }

  function _sort(uint256[] memory a, function(uint256, uint256) pure returns (bool) comp) private pure returns (uint256[] memory) {
    uint256 begin;
    uint256 end;
    assembly ("memory-safe") {
      begin := add(a, 0x20)
      end := add(begin, mul(mload(a), 0x20))
    }
    _quickSort(begin, end, comp);
    return a;
  }

  function _lt(uint256 a, uint256 b) private pure returns (bool) {
    return a < b;
  }

  function _gt(uint256 a, uint256 b) private pure returns (bool) {
    return a > b;
  }

  // Same-signature dispatch candidates with loops of their own.
  function _upperBound(uint256 x, uint256 n) private pure returns (bool) {
    uint256 low = 0;
    uint256 high = n;
    while (low < high) {
      uint256 mid = (low & high) + (low ^ high) / 2;
      if (mid * mid > x) high = mid;
      else low = mid + 1;
    }
    return low > 3;
  }

  function _gcdOdd(uint256 a, uint256 b) private pure returns (bool) {
    while (b != 0) {
      (a, b) = (b, a % b);
    }
    return a % 2 == 1;
  }

  function _sortedPair(uint256 a, uint256 b) private pure returns (bool) {
    uint256[] memory pair = new uint256[](2);
    pair[0] = b;
    pair[1] = a;
    pair = _sort(pair, _lt);
    return pair[0] <= pair[1];
  }

  function sort(uint256[] memory a) public pure returns (uint256[] memory) {
    return _sort(a, _lt);
  }

  function sortReverse(uint256[] memory a) public pure returns (uint256[] memory) {
    return _sort(a, _gt);
  }

  function misc(uint256 a, uint256 b) public pure returns (bool, bool, bool) {
    return (_upperBound(a, b), _gcdOdd(a, b), _sortedPair(a, b));
  }
}
