/*
 * The C++ allocation functions for mimalloc on Windows.
 *
 * Windows programs take `malloc` from the C runtime DLL, which a static link
 * cannot replace. The replaceable `operator new` and `operator delete` move
 * the LLVM and MLIR C++ allocations to mimalloc instead.
 */

#include <mimalloc-new-delete.h>
