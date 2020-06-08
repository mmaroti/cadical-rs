#include "../cadical/src/internal.hpp"

#include <chrono>

/*------------------------------------------------------------------------*/

// Generic c++11 implementation

namespace CaDiCaL {

double absolute_real_time () {
  return std::chrono::duration_cast<std::chrono::microseconds>(
    std::chrono::system_clock::now().time_since_epoch()).count() * 1e-6;
}

double Internal::real_time () {
  return absolute_real_time () - stats.time.real;
}

double absolute_process_time () {
  return absolute_real_time();
}

double Internal::process_time () {
  return absolute_process_time () - stats.time.process;
}

uint64_t current_resident_set_size () {
  return 0;
}

}
