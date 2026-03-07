#include "miyabi/miyabi.h"
#include "miyabi/bridge.h"

extern "C" MiyabiVTable get_miyabi_vtable();

int main() {
    init_engine_systems();

    MiyabiVTable vtable = get_miyabi_vtable();
    if (vtable.abi_version != MIYABI_ABI_VERSION) {
        shutdown_engine_systems();
        return 1;
    }
    Game* game = vtable.create_game();

    step_engine_systems();
    vtable.update_game(game);

    vtable.destroy_game(game);
    shutdown_engine_systems();
    return 0;
}
