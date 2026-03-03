#include "renderer/RenderBatching.hpp"

#include <algorithm>

void sort_renderables_for_batching(std::vector<RenderableObject>& renderables) {
    std::sort(
        renderables.begin(),
        renderables.end(),
        [](const RenderableObject& lhs, const RenderableObject& rhs) {
            if (lhs.material_id != rhs.material_id) {
                return lhs.material_id < rhs.material_id;
            }
            return lhs.mesh_id < rhs.mesh_id;
        });
}

std::vector<MaterialMeshBatch> build_material_mesh_batches(
    const std::vector<RenderableObject>& sorted_renderables) {
    std::vector<MaterialMeshBatch> batches;
    if (sorted_renderables.empty()) {
        return batches;
    }

    size_t batch_start = 0;
    uint32_t current_material = sorted_renderables[0].material_id;
    uint32_t current_mesh = sorted_renderables[0].mesh_id;

    for (size_t i = 1; i < sorted_renderables.size(); ++i) {
        const auto& item = sorted_renderables[i];
        if (item.material_id == current_material && item.mesh_id == current_mesh) {
            continue;
        }

        batches.push_back(MaterialMeshBatch{
            current_material,
            current_mesh,
            batch_start,
            i - batch_start,
        });

        batch_start = i;
        current_material = item.material_id;
        current_mesh = item.mesh_id;
    }

    batches.push_back(MaterialMeshBatch{
        current_material,
        current_mesh,
        batch_start,
        sorted_renderables.size() - batch_start,
    });

    return batches;
}
