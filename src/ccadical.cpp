#include "../cadical/src/ccadical.cpp"

// This files converts some of the C++ interface of cadical to C.
// These functions are not available in the C interface of cadical.
// The C interface that cadical provides is in: cadical/src/ccadical.h & cadical/src/ccadical.cpp

extern "C"
{

  int ccadical_status(CCaDiCaL *wrapper)
  {
    return ((Wrapper *)wrapper)->solver->status();
  }

  int ccadical_vars(CCaDiCaL *wrapper)
  {
    return ((Wrapper *)wrapper)->solver->vars();
  }

  const char *ccadical_read_dimacs(CCaDiCaL *wrapper, const char *path,
                                   int &vars, int strict)
  {
    return ((Wrapper *)wrapper)->solver->read_dimacs(path, vars, strict);
  }

  const char *ccadical_write_dimacs(CCaDiCaL *wrapper, const char *path,
                                    int min_max_var = 0)
  {
    return ((Wrapper *)wrapper)->solver->write_dimacs(path, min_max_var);
  }

  int ccadical_configure(CCaDiCaL *wrapper, const char *name)
  {
    return ((Wrapper *)wrapper)->solver->configure(name);
  }

  int ccadical_limit2(CCaDiCaL *wrapper,
                      const char *name, int val)
  {
    return ((Wrapper *)wrapper)->solver->limit(name, val);
  }
}
