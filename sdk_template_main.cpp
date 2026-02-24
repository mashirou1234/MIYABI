#include "miyabi/miyabi.h"
#include "miyabi/bridge.h"

extern "C" MiyabiVTable get_miyabi_vtable();

int main() {
    init_engine_systems();

    MiyabiVTable vtable = get_miyabi_vtable();
    if (vtable.abi_version != MIYABI_ABI_VERSION) {
        return 1;
    }
    Game* game = vtable.create_game();

    step_engine_systems();
    vtable.update_game(game);

    vtable.destroy_game(game);
    return 0;
}
