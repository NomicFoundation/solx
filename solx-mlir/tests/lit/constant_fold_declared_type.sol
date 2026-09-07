// RUN: solx --emit-mlir=sol %constants/constant_fold_declared_type.sol | FileCheck %s
// RUN: solc --mlir-action=print-init %constants/constant_fold_declared_type.sol 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*digits.*}}() -> !sol.fixedbytes<16>
// CHECK:   %[[DGC:.*]] = sol.constant 64058384521018188869745042196707698022 : ui128
// CHECK:   %[[DG:.*]] = sol.bytes_cast %[[DGC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.return %[[DG]] : !sol.fixedbytes<16>

// CHECK: sol.func @{{.*hexDigit.*}} -> !sol.fixedbytes<1>
// CHECK:   %[[HDC:.*]] = sol.constant 64058384521018188869745042196707698022 : ui128
// CHECK:   %[[HD:.*]] = sol.bytes_cast %[[HDC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.fixed_bytes_index %[[HD]][%{{.*}}] : !sol.fixedbytes<16>, ui256 -> !sol.fixedbytes<1>

// CHECK: sol.func @{{.*pubDigit.*}} -> !sol.fixedbytes<1>
// CHECK:   %[[PDC:.*]] = sol.constant 64058384521018188869745006874357679430 : ui128
// CHECK:   %[[PD:.*]] = sol.bytes_cast %[[PDC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.fixed_bytes_index %[[PD]][%{{.*}}] : !sol.fixedbytes<16>, ui256 -> !sol.fixedbytes<1>

// CHECK: sol.func @{{.*fileDigit.*}} -> !sol.fixedbytes<1>
// CHECK:   %[[FDC:.*]] = sol.constant 92071172066227433993572960279997788208 : ui128
// CHECK:   %[[FD:.*]] = sol.bytes_cast %[[FDC]] : ui128 to !sol.fixedbytes<16>
// CHECK:   sol.fixed_bytes_index %[[FD]][%{{.*}}] : !sol.fixedbytes<16>, ui256 -> !sol.fixedbytes<1>

// CHECK: sol.func @{{.*text.*}}() -> !sol.string<Memory>
// CHECK:   %[[TX:.*]] = sol.string_lit "solx" -> !sol.string<Memory>
// CHECK:   sol.return %[[TX]] : !sol.string<Memory>
