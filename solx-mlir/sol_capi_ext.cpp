//===- sol_capi_ext.cpp - C wrappers for Sol dialect attrs and types ------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
// The Sol dialect C API (mlir-c/Dialect/Sol.h) does not expose constructors for
// several attributes (e.g. ContractKindAttr, StateMutabilityAttr) or types
// (e.g. PointerType, AddressType, ContractType). These thin wrappers call the
// generated C++ get() methods via extern "C" linkage so Rust can create them
// through FFI.
//
//===----------------------------------------------------------------------===//

#include "mlir/Dialect/Sol/Sol.h"
#include "mlir/IR/BuiltinTypes.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir-c/BuiltinAttributes.h"
#include "mlir-c/IR.h"
#include "mlir/CAPI/IR.h"
#include "llvm/ADT/APInt.h"
#include "llvm/ADT/ArrayRef.h"

#include <cstddef>
#include <cstdint>
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

MlirAttribute solxCreateIntegerAttr(MlirType ty, bool isNegative,
                                    size_t numWords, const uint64_t *magnitude) {
    unsigned bitWidth = unwrap(ty).getIntOrFloatBitWidth();
    llvm::APInt value = numWords == 0
        ? llvm::APInt::getZero(bitWidth)
        : llvm::APInt(bitWidth, llvm::ArrayRef<uint64_t>(magnitude, numWords));
    if (isNegative) value.negate();
    return mlirIntegerAttrGetFromWords(ty, value.getNumWords(), value.getRawData());
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

MlirType solxCreateContractType(MlirContext ctx, const char *namePtr,
                                size_t nameLen, bool payable) {
    auto *context = unwrap(ctx);
    llvm::StringRef name(namePtr, nameLen);
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

MlirType solxCreateByteType(MlirContext ctx) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::ByteType::get(context));
}

MlirType solxCreateArrayType(MlirContext ctx, int64_t size, MlirType elementType,
                             uint32_t dataLocation) {
    if (dataLocation > 5) abort();
    auto *context = unwrap(ctx);
    auto location = static_cast<mlir::sol::DataLocation>(dataLocation);
    return wrap(mlir::sol::ArrayType::get(context, size, unwrap(elementType), location));
}

MlirType solxCreateMappingType(MlirContext ctx, MlirType keyType, MlirType valueType) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::MappingType::get(context, unwrap(keyType), unwrap(valueType)));
}

MlirType solxCreateStructType(MlirContext ctx, const MlirType *memberTypes,
                              size_t memberCount, uint32_t dataLocation) {
    if (dataLocation > 5) abort();
    auto *context = unwrap(ctx);
    std::vector<mlir::Type> members;
    members.reserve(memberCount);
    for (size_t i = 0; i < memberCount; i++) {
        members.push_back(unwrap(memberTypes[i]));
    }
    auto location = static_cast<mlir::sol::DataLocation>(dataLocation);
    return wrap(mlir::sol::StructType::get(context, members, location));
}

MlirType solxCreateEnumType(MlirContext ctx, uint32_t max) {
    auto *context = unwrap(ctx);
    return wrap(mlir::sol::EnumType::get(context, max));
}

MlirType solxCreateFuncRefType(MlirContext ctx, const MlirType *paramTypes,
                               size_t paramCount, const MlirType *resultTypes,
                               size_t resultCount) {
    auto *context = unwrap(ctx);
    std::vector<mlir::Type> params;
    params.reserve(paramCount);
    for (size_t i = 0; i < paramCount; i++) {
        params.push_back(unwrap(paramTypes[i]));
    }
    std::vector<mlir::Type> results;
    results.reserve(resultCount);
    for (size_t i = 0; i < resultCount; i++) {
        results.push_back(unwrap(resultTypes[i]));
    }
    auto fnTy = mlir::FunctionType::get(context, params, results);
    return wrap(mlir::sol::FuncRefType::get(context, fnTy));
}

MlirType solxCreateExtFuncRefType(MlirContext ctx, const MlirType *paramTypes,
                                  size_t paramCount,
                                  const MlirType *resultTypes,
                                  size_t resultCount) {
    auto *context = unwrap(ctx);
    std::vector<mlir::Type> params;
    params.reserve(paramCount);
    for (size_t i = 0; i < paramCount; i++) {
        params.push_back(unwrap(paramTypes[i]));
    }
    std::vector<mlir::Type> results;
    results.reserve(resultCount);
    for (size_t i = 0; i < resultCount; i++) {
        results.push_back(unwrap(resultTypes[i]));
    }
    auto fnTy = mlir::FunctionType::get(context, params, results);
    return wrap(mlir::sol::ExtFuncRefType::get(context, fnTy));
}

/*
 * Type predicates.
 *
 * Typed `isa<>` introspection for Sol-dialect types, never textual AsmPrinter
 * matching (which silently miscompiles if the type printer drifts). One
 * predicate per Sol type; the Rust side composes categories (reference,
 * address-like).
 */

bool solxIsEnumType(MlirType ty) {
    return mlir::isa<mlir::sol::EnumType>(unwrap(ty));
}

bool solxIsAddressType(MlirType ty) {
    return mlir::isa<mlir::sol::AddressType>(unwrap(ty));
}

bool solxIsContractType(MlirType ty) {
    return mlir::isa<mlir::sol::ContractType>(unwrap(ty));
}

bool solxIsFixedBytesType(MlirType ty) {
    return mlir::isa<mlir::sol::FixedBytesType>(unwrap(ty));
}

uint32_t solxFixedBytesTypeSize(MlirType ty) {
    return mlir::cast<mlir::sol::FixedBytesType>(unwrap(ty)).getSize();
}

bool solxIsByteType(MlirType ty) {
    return mlir::isa<mlir::sol::ByteType>(unwrap(ty));
}

bool solxIsStringType(MlirType ty) {
    return mlir::isa<mlir::sol::StringType>(unwrap(ty));
}

bool solxIsArrayType(MlirType ty) {
    return mlir::isa<mlir::sol::ArrayType>(unwrap(ty));
}

bool solxIsStructType(MlirType ty) {
    return mlir::isa<mlir::sol::StructType>(unwrap(ty));
}

bool solxIsMappingType(MlirType ty) {
    return mlir::isa<mlir::sol::MappingType>(unwrap(ty));
}

bool solxIsExtFuncRefType(MlirType ty) {
    return mlir::isa<mlir::sol::ExtFuncRefType>(unwrap(ty));
}

bool solxIsFuncRefType(MlirType ty) {
    return mlir::isa<mlir::sol::FuncRefType>(unwrap(ty));
}

bool solxIsPointerType(MlirType ty) {
    return mlir::isa<mlir::sol::PointerType>(unwrap(ty));
}

/*
 * Pointer type accessors.
 *
 * A `!sol.ptr<T, Loc>` carries its pointee type and data location; the Rust
 * `Pointer` entity reads both from its own type instead of threading them
 * alongside the value. The caller must ensure `ty` is a pointer type.
 */

MlirType solxPointerTypePointeeType(MlirType ty) {
    return wrap(mlir::cast<mlir::sol::PointerType>(unwrap(ty)).getPointeeType());
}

uint32_t solxPointerTypeDataLocation(MlirType ty) {
    return static_cast<uint32_t>(
        mlir::cast<mlir::sol::PointerType>(unwrap(ty)).getDataLocation());
}

// A string/array/struct carries its own data location; a mapping carries none.
uint32_t solxReferenceTypeDataLocation(MlirType ty) {
    auto type = unwrap(ty);
    if (auto string = mlir::dyn_cast<mlir::sol::StringType>(type))
        return static_cast<uint32_t>(string.getDataLocation());
    if (auto array = mlir::dyn_cast<mlir::sol::ArrayType>(type))
        return static_cast<uint32_t>(array.getDataLocation());
    if (auto structure = mlir::dyn_cast<mlir::sol::StructType>(type))
        return static_cast<uint32_t>(structure.getDataLocation());
    abort();
}

} /* extern "C" */
