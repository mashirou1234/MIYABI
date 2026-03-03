#pragma once

#include <cstddef>
#include <cstdint>
#include <vector>

#include "miyabi/miyabi.h"

struct MaterialMeshBatch {
    uint32_t material_id;
    uint32_t mesh_id;
    size_t start_index;
    size_t instance_count;
};

void sort_renderables_for_batching(std::vector<RenderableObject>& renderables);
std::vector<MaterialMeshBatch> build_material_mesh_batches(
    const std::vector<RenderableObject>& sorted_renderables);
