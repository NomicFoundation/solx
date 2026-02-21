/*
 * Stub definitions for MLIR C API ExecutionEngine symbols.
 *
 * The `melior` crate unconditionally compiles its `execution_engine` module,
 * which references six symbols from `libMLIRCAPIExecutionEngine.a`. That
 * library is only built when CMake's MLIR_ENABLE_EXECUTION_ENGINE is ON,
 * which requires a native target in LLVM_TARGETS_TO_BUILD. solx builds
 * only the EVM experimental target, so the library is never produced.
 *
 * On ELF (Linux) and Mach-O (macOS) the linker silently skips the
 * unreferenced archive member, but on PE/COFF (Windows/MinGW) `ld.lld`
 * pulls it in and fails with undefined-symbol errors.
 *
 * These stubs provide the six symbols so the linker succeeds on every
 * platform. solx never calls melior::ExecutionEngine at runtime, so the
 * abort() bodies are purely defensive.
 *
 * If melior gains a feature gate for ExecutionEngine, these stubs can be
 * removed. Track: https://github.com/mlir-rs/melior/issues
 */

#include <stdint.h>
#include <stdlib.h>

/* Opaque stand-ins — the real types live in mlir-c/ExecutionEngine.h,
   but we only need ABI-compatible signatures for the linker. */
struct MlirExecutionEngine { void *ptr; };
struct MlirStringRef        { const char *data; size_t length; };
struct MlirLogicalResult    { int8_t value; };
struct MlirModule           { void *ptr; };

struct MlirExecutionEngine mlirExecutionEngineCreate(
    struct MlirModule op, int optLevel, int numPaths,
    const struct MlirStringRef *sharedLibPaths, _Bool enableObjectDump)
{
    (void)op; (void)optLevel; (void)numPaths;
    (void)sharedLibPaths; (void)enableObjectDump;
    abort();
}

void mlirExecutionEngineDestroy(struct MlirExecutionEngine jit)
{
    (void)jit;
    abort();
}

struct MlirLogicalResult mlirExecutionEngineInvokePacked(
    struct MlirExecutionEngine jit, struct MlirStringRef name, void **arguments)
{
    (void)jit; (void)name; (void)arguments;
    abort();
}

void *mlirExecutionEngineLookup(
    struct MlirExecutionEngine jit, struct MlirStringRef name)
{
    (void)jit; (void)name;
    abort();
}

void mlirExecutionEngineRegisterSymbol(
    struct MlirExecutionEngine jit, struct MlirStringRef name, void *sym)
{
    (void)jit; (void)name; (void)sym;
    abort();
}

void mlirExecutionEngineDumpToObjectFile(
    struct MlirExecutionEngine jit, struct MlirStringRef fileName)
{
    (void)jit; (void)fileName;
    abort();
}
