#include <cstdint>

// This is a simple implementation for the performance test.
// In the future, this could read from a config file or be controlled by user input.
extern "C" uint32_t get_performance_test_sprite_count() {
    return 10000;
}