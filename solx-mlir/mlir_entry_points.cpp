/*
 * The upstream MLIR entry points solx uses, wrapped here because their C API
 * libraries (MLIRCAPIRegisterEverything, MLIRCAPIConversion, MLIRCAPITarget)
 * link every dialect, conversion and translation. Calling these instead lets
 * CI install only the MLIR libraries solx links (`--trim-libraries`).
 */
#include "mlir/CAPI/IR.h"
#include "mlir/CAPI/Pass.h"
#include "mlir/Conversion/ArithToLLVM/ArithToLLVM.h"
#include "mlir/Conversion/ControlFlowToLLVM/ControlFlowToLLVM.h"
#include "mlir/Conversion/FuncToLLVM/ConvertFuncToLLVMPass.h"
#include "mlir/Conversion/ReconcileUnrealizedCasts/ReconcileUnrealizedCasts.h"
#include "mlir/Conversion/SCFToControlFlow/SCFToControlFlow.h"
#include "mlir/Conversion/SolToYul/SolToYul.h"
#include "mlir/Conversion/YulToStandard/YulToStandard.h"
#include "mlir/Dialect/Arith/IR/Arith.h"
#include "mlir/Dialect/ControlFlow/IR/ControlFlow.h"
#include "mlir/Dialect/Func/Extensions/InlinerExtension.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/Dialect/LLVMIR/LLVMDialect.h"
#include "mlir/Dialect/LLVMIR/Transforms/InlinerInterfaceImpl.h"
#include "mlir/Dialect/SCF/IR/SCF.h"
#include "mlir/Dialect/UB/IR/UBOps.h"
#include "mlir/IR/DialectRegistry.h"
#include "mlir/Target/LLVMIR/Dialect/Builtin/BuiltinToLLVMIRTranslation.h"
#include "mlir/Target/LLVMIR/Dialect/LLVMIR/LLVMToLLVMIRTranslation.h"
#include "mlir/Target/LLVMIR/Export.h"
#include "llvm-c/Core.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"

extern "C" {

void solxRegisterDialects(MlirDialectRegistry registry) {
    auto &dialects = *unwrap(registry);
    dialects.insert<mlir::arith::ArithDialect, mlir::cf::ControlFlowDialect,
                    mlir::func::FuncDialect, mlir::LLVM::LLVMDialect,
                    mlir::scf::SCFDialect, mlir::ub::UBDialect>();
    mlir::func::registerInlinerExtension(dialects);
    mlir::LLVM::registerInlinerInterface(dialects);
}

void solxRegisterLLVMTranslations(MlirContext context) {
    mlir::DialectRegistry registry;
    mlir::registerBuiltinDialectTranslation(registry);
    mlir::registerLLVMDialectTranslation(registry);
    unwrap(context)->appendDialectRegistry(registry);
}

LLVMModuleRef solxTranslateModuleToLLVMIR(MlirOperation module, LLVMContextRef context) {
    std::unique_ptr<llvm::Module> llvmModule =
        mlir::translateModuleToLLVMIR(unwrap(module), *llvm::unwrap(context));
    return llvm::wrap(llvmModule.release());
}

MlirPass solxCreateConvertSolToYulPass() {
    return wrap(mlir::createConvertSolToYulPass().release());
}

MlirPass solxCreateConvertYulToStandardPass() {
    return wrap(mlir::createConvertYulToStandardPass().release());
}

MlirPass solxCreateSCFToControlFlowPass() {
    return wrap(mlir::createSCFToControlFlowPass().release());
}

MlirPass solxCreateConvertFuncToLLVMPass() {
    return wrap(mlir::createConvertFuncToLLVMPass().release());
}

MlirPass solxCreateArithToLLVMConversionPass() {
    return wrap(mlir::createArithToLLVMConversionPass().release());
}

MlirPass solxCreateConvertControlFlowToLLVMPass() {
    return wrap(mlir::createConvertControlFlowToLLVMPass().release());
}

MlirPass solxCreateReconcileUnrealizedCastsPass() {
    return wrap(mlir::createReconcileUnrealizedCastsPass().release());
}

}
