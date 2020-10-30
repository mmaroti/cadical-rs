// Disable asserts because one of the contains a closure
#include "../../cadical/src/internal.hpp"
#define assert(A) ((void)0)
#include "../../cadical/src/lookahead.cpp"
