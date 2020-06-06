#include "../cadical/src/ccadical.h"
#include "../cadical/src/cadical.hpp"

namespace CaDiCaL
{
    struct Wrapper : Terminator
    {
        Solver *solver;
        void *state;
        int (*function)(void *);
        bool terminate() { return function ? function(state) : false; }
        Wrapper() : solver(new Solver()), state(0), function(0) {}
        ~Wrapper()
        {
            function = 0;
            delete solver;
        }
    };
} // namespace CaDiCaL

extern "C"
{
    int ccadical_vars(CCaDiCaL * wrapper)
    {
        return ((CaDiCaL::Wrapper *)wrapper)->solver->vars();
    }
}
