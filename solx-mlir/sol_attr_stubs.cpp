/*
 * C wrappers for Sol dialect attribute and type creation.
 *
 * The Sol dialect's C API (mlir-c/Dialect/Sol.h) does not expose
 * constructors for several attributes (e.g. ContractKindAttr,
 * StateMutabilityAttr) or types (e.g. PointerType, AddressType,
 * ContractType). These thin wrappers call the generated C++ get()
 * methods via extern "C" linkage so Rust can create them through FFI.
 */

#include "mlir/Dialect/Sol/Sol.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir-c/IR.h"
#include "mlir/CAPI/IR.h"

#include <cstdlib>
#include <vector>

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

MlirAttribute solxCreateFunctionKindAttr(MlirContext ctx, uint32_t kind) {
    if (kind > 2) abort();
    auto *context = unwrap(ctx);
    auto attr = mlir::sol::FunctionKindAttr::get(
        context, static_cast<mlir::sol::FunctionKind>(kind));
    return wrap(attr);
}

MlirAttribute solxCreateEvmVersionAttr(MlirContext ctx, uint32_t version) {
    if (version < 11 || version > 13) abort();
    auto *context = unwrap(ctx);
    auto attr = mlir::sol::EvmVersionAttr::get(
        context, static_cast<mlir::sol::EvmVersion>(version));
    return wrap(attr);
}

MlirType solxCreatePointerType(MlirContext ctx, MlirType elementType, uint32_t dataLocation) {
    if (dataLocation > 5) abort();
    auto *context = unwrap(ctx);
    auto elemType = unwrap(elementType);
    auto location = static_cast<mlir::sol::DataLocation>(dataLocation);
    return wrap(mlir::sol::PointerType::get(context, elemType, location));
}

MlirType solxCreateAddressType(MlirContext ctx, bool payable) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::AddressType::get(context, payable));
}

MlirType solxCreateContractType(MlirContext ctx, const char *name_ptr,
                                size_t name_len, bool payable) {
    auto *context = unwrap(ctx);
    llvm::StringRef name(name_ptr, name_len);
    return wrap(mlir::sol::ContractType::get(context, name, payable));
}

MlirType solxCreateStringType(MlirContext ctx, uint32_t dataLocation) {
    if (dataLocation > 5) abort();
    auto *context = unwrap(ctx);
    auto location = static_cast<mlir::sol::DataLocation>(dataLocation);
    return wrap(mlir::sol::StringType::get(context, location));
}

MlirType solxCreateFixedBytesType(MlirContext ctx, uint32_t size) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::FixedBytesType::get(context, size));
}

MlirType solxCreateArrayType(MlirContext ctx, int64_t size, MlirType elementType,
                             uint32_t dataLocation) {
    if (dataLocation > 5) abort();
    auto *context = unwrap(ctx);
    auto location = static_cast<mlir::sol::DataLocation>(dataLocation);
    return wrap(mlir::sol::ArrayType::get(context, size, unwrap(elementType), location));
}

MlirType solxCreateMappingType(MlirContext ctx, MlirType keyType, MlirType valType) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::MappingType::get(context, unwrap(keyType), unwrap(valType)));
}

MlirType solxCreateStructType(MlirContext ctx, const MlirType *member_types,
                              size_t member_count, uint32_t dataLocation) {
    if (dataLocation > 5) abort();
    auto *context = unwrap(ctx);
    std::vector<mlir::Type> mems;
    mems.reserve(member_count);
    for (size_t i = 0; i < member_count; i++) {
        mems.push_back(unwrap(member_types[i]));
    }
    auto location = static_cast<mlir::sol::DataLocation>(dataLocation);
    return wrap(mlir::sol::StructType::get(context, mems, location));
}

MlirType solxCreateEnumType(MlirContext ctx, uint32_t max) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::EnumType::get(context, max));
}

} /* extern "C" */
