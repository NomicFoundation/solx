// RUN: cd %S && solx --emit-mlir=sol import_relative_curdir.sol | FileCheck import_relative_curdir.sol
// RUN: cd %S && solc --mlir-action=print-init import_relative_curdir.sol 2>/dev/null | FileCheck import_relative_curdir.sol

// A leading `./` import resolves against the importing file's directory: the self-import below
// folds back to this file, exercising the relative-path normalization that drops the `.` segment.
// Run from the file's own directory so its identifier stays a bare name and the leading `.` survives.

import "./import_relative_curdir.sol";

contract CurDirImport {
    function value() public pure returns (uint256) {
        return 42;
    }
}

// CHECK: sol.contract @CurDirImport
// CHECK: sol.constant 42 : ui8
// FIX: must be above code
