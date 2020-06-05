#include "../cadical/src/version.hpp"

extern "C"
{
    const char *cadical_version() { return CaDiCaL::version(); }
}
