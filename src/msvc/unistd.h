// Only for the MSVC compiler
#ifdef _MSC_VER

#include <io.h>
#include <intrin.h>

#define pclose _pclose
#define popen _popen
#define access _access
#define isatty _isatty

#define R_OK 4
#define W_OK 2
#define S_ISDIR(mode)  (((mode) & S_IFMT) == S_IFDIR)

#define __PRETTY_FUNCTION__ __FUNCTION__
#define  __builtin_prefetch(A,B,C) _m_prefetch((void*)(A))

#endif
