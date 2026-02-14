#include "miyabi/bridge.h"

#ifdef MIYABI_PERFORMANCE_TEST
#include <cstdint>
uint32_t g_performance_test_sprite_count = 50000;
uint32_t get_performance_test_sprite_count() {
    return g_performance_test_sprite_count;
}
#endif
