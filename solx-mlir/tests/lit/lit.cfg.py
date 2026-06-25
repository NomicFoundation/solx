import os
import lit.formats

config.name = "solx-mlir"
config.test_format = lit.formats.ShTest(True)
config.suffixes = [".sol"]

config_dir = os.path.dirname(os.path.abspath(__file__))
solx_root = os.path.join(config_dir, "..", "..", "..")
solx_bin_dir = os.path.join(solx_root, "target-slang", os.environ.get("SOLX_LIT_TARGET", ""), "debug")
solc_bin_dir = os.path.join(solx_root, "solx-solidity", "build", "solc")

config.environment["PATH"] = os.pathsep.join(
    [solx_bin_dir, solc_bin_dir, os.environ.get("PATH", "")]
)

# Propagate the coverage profile path so each solx invocation spawned by lit
# writes its own .profraw file (used for LIT-only coverage measurement). No-op
# when LLVM_PROFILE_FILE is unset, i.e. for ordinary non-instrumented runs.
if os.environ.get("LLVM_PROFILE_FILE"):
    config.environment["LLVM_PROFILE_FILE"] = os.environ["LLVM_PROFILE_FILE"]

config.test_source_root = config_dir
config.test_exec_root = os.path.join(config_dir, "Output")
