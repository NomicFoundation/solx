import os
import lit.formats

config.name = "solx-mlir"
config.test_format = lit.formats.ShTest(True)
config.suffixes = [".sol"]

config_dir = os.path.dirname(os.path.abspath(__file__))
solx_root = os.path.join(config_dir, "..", "..", "..")
solx_bin_dir = os.path.join(solx_root, "target-slang", os.environ.get("SOLX_LIT_TARGET", ""), "debug")

config.environment["PATH"] = solx_bin_dir + os.pathsep + os.environ.get("PATH", "")

config.test_source_root = config_dir
config.test_exec_root = os.path.join(config_dir, "Output")
