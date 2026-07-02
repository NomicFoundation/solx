// RUN: cd %S && solx --emit-mlir=sol import_relative_curdir.sol | FileCheck import_relative_curdir.sol
// RUN: cd %S && solc --mlir-action=print-init import_relative_curdir.sol 2>/dev/null | FileCheck import_relative_curdir.sol

// CHECK: sol.contract @CurrentDirectoryImport
// CHECK: sol.constant 42 : ui8

import "./import_relative_curdir.sol";

contract CurrentDirectoryImport {
    function value() public pure returns (uint256) {
        return 42;
    }
}
