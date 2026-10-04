/*
 * MLIR pass timing handed to Rust.
 *
 * The MLIR C API's only pass timing, mlirPassManagerEnableTiming, prints its
 * report to stderr. This installs MLIR's DefaultTimingManager with an output
 * strategy that passes each report entry to a callback instead.
 */

#include "mlir/Pass/PassManager.h"
#include "mlir/Support/Timing.h"
#include "mlir-c/Pass.h"
#include "mlir/CAPI/Pass.h"
#include "mlir/CAPI/Support.h"
#include "llvm/Support/raw_ostream.h"

extern "C" {

typedef void (*SolxPassTimingCallback)(MlirStringRef name, uint32_t depth,
                                       double wallSeconds, void *userData);

} /* extern "C" */

namespace {

/// Hands every entry of a pass timing report to a callback.
class CallbackOutputStrategy final : public mlir::OutputStrategy {
public:
    CallbackOutputStrategy(SolxPassTimingCallback callback, void *userData)
        : mlir::OutputStrategy(llvm::nulls()), callback(callback),
          userData(userData) {}

    void printHeader(const mlir::TimeRecord &) override {}
    void printFooter() override {}
    void printTime(const mlir::TimeRecord &, const mlir::TimeRecord &) override {}

    // In tree mode the list entries are the report's closing `Rest` and `Total` rows.
    void printListEntry(llvm::StringRef name, const mlir::TimeRecord &time,
                        const mlir::TimeRecord &, bool) override {
        callback(wrap(name), 0, time.wall, userData);
    }

    void printTreeEntry(unsigned indent, llvm::StringRef name,
                        const mlir::TimeRecord &time,
                        const mlir::TimeRecord &) override {
        // The tree printer indents each nesting level by two.
        callback(wrap(name), indent / 2, time.wall, userData);
    }

    void printTreeEntryEnd(unsigned, bool) override {}

private:
    SolxPassTimingCallback callback;
    void *userData;
};

} // namespace

extern "C" {

void solxPassManagerEnableTiming(MlirPassManager pm,
                                 SolxPassTimingCallback callback,
                                 void *userData) {
    auto tm = std::make_unique<mlir::DefaultTimingManager>();
    tm->setEnabled(true);
    tm->setOutput(std::make_unique<CallbackOutputStrategy>(callback, userData));
    unwrap(pm)->enableTiming(std::move(tm));
}

} /* extern "C" */
