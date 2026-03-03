#include <cassert>
#include <utility>
#include <vector>

#include "renderer/RenderBatching.hpp"

int main() {
    const auto make_renderable =
        [](uint32_t texture_id, uint32_t mesh_id, uint32_t material_id) {
            RenderableObject object{};
            object.texture_id = texture_id;
            object.mesh_id = mesh_id;
            object.material_id = material_id;
            return object;
        };

    std::vector<RenderableObject> renderables = {
        make_renderable(4, 3, 2),
        make_renderable(2, 1, 1),
        make_renderable(5, 2, 1),
        make_renderable(6, 1, 1),
        make_renderable(7, 2, 2),
        make_renderable(8, 2, 1),
        make_renderable(9, 3, 2),
    };

    sort_renderables_for_batching(renderables);

    const std::vector<std::pair<uint32_t, uint32_t>> expected_order = {
        {1, 1},
        {1, 1},
        {1, 2},
        {1, 2},
        {2, 2},
        {2, 3},
        {2, 3},
    };

    assert(renderables.size() == expected_order.size());
    for (size_t i = 0; i < renderables.size(); ++i) {
        assert(renderables[i].material_id == expected_order[i].first);
        assert(renderables[i].mesh_id == expected_order[i].second);
    }

    const std::vector<MaterialMeshBatch> batches =
        build_material_mesh_batches(renderables);
    assert(batches.size() == 4);

    assert(batches[0].material_id == 1);
    assert(batches[0].mesh_id == 1);
    assert(batches[0].start_index == 0);
    assert(batches[0].instance_count == 2);

    assert(batches[1].material_id == 1);
    assert(batches[1].mesh_id == 2);
    assert(batches[1].start_index == 2);
    assert(batches[1].instance_count == 2);

    assert(batches[2].material_id == 2);
    assert(batches[2].mesh_id == 2);
    assert(batches[2].start_index == 4);
    assert(batches[2].instance_count == 1);

    assert(batches[3].material_id == 2);
    assert(batches[3].mesh_id == 3);
    assert(batches[3].start_index == 5);
    assert(batches[3].instance_count == 2);

    return 0;
}
