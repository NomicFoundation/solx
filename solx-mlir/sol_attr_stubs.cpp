/*
 * C wrappers for Sol dialect attribute creation.
 *
 * The Sol dialect's C API (mlir-c/Dialect/Sol.h) does not expose
 * ContractKindAttr or StateMutabilityAttr constructors. These thin
 * wrappers call the generated C++ get() methods via extern "C" linkage
 * so Rust can create these attributes through FFI.
 */

#include "mlir/Dialect/Sol/Sol.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir-c/IR.h"
#include "mlir/CAPI/IR.h"

#include <cstdlib>

extern "C" {

MlirAttribute solxCreateContractKindAttr(MlirContext ctx, uint32_t kind) {
    if (kind > 2) abort();
    auto *context = unwrap(ctx);
    auto attr = mlir::sol::ContractKindAttr::get(
        context, static_cast<mlir::sol::ContractKind>(kind));
    return wrap(attr);
}

MlirAttribute solxCreateStateMutabilityAttr(MlirContext ctx, uint32_t mutability) {
    if (mutability > 3) abort();
    auto *context = unwrap(ctx);
    auto attr = mlir::sol::StateMutabilityAttr::get(
        context, static_cast<mlir::sol::StateMutability>(mutability));
    return wrap(attr);
}

MlirAttribute solxCreateEvmVersionAttr(MlirContext ctx, uint32_t version) {
    if (version < 11 || version > 13) abort();
    auto *context = unwrap(ctx);
    auto attr = mlir::sol::EvmVersionAttr::get(
        context, static_cast<mlir::sol::EvmVersion>(version));
    return wrap(attr);
}

} /* extern "C" */
