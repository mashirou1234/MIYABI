use crate::{ffi, ComponentType, InternalWorld, Material, Sprite};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfConfig {
    pub warmup_iterations: u32,
    pub iterations: u32,
    pub sprite_count: usize,
    pub ui_items_per_row: usize,
    pub ui_items_per_col: usize,
    pub scene_entity_count: usize,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 5,
            iterations: 30,
            sprite_count: 10_000,
            ui_items_per_row: 30,
            ui_items_per_col: 40,
            scene_entity_count: 5_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfScenarioResult {
    pub name: String,
    pub avg_ms: f64,
    pub p95_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfReport {
    pub schema_version: u32,
    pub generated_unix_epoch_sec: u64,
    pub config: PerfConfig,
    pub scenarios: Vec<PerfScenarioResult>,
}

pub fn run_performance_baseline(config: PerfConfig) -> PerfReport {
    let sprite_world = build_sprite_world(config.sprite_count);
    let texture_map = build_texture_map(config.sprite_count);

    let sprite_samples = benchmark_samples(config.warmup_iterations, config.iterations, || {
        let _renderables = build_renderables_from_world(&sprite_world, &texture_map);
    });

    let ui_samples = benchmark_samples(config.warmup_iterations, config.iterations, || {
        let _text_commands = build_ui_text_commands(config.ui_items_per_row, config.ui_items_per_col);
    });

    let scene_samples = benchmark_samples(config.warmup_iterations, config.iterations, || {
        let _ = run_scene_construct_destruct_cycle(config.scene_entity_count);
    });

    let generated_unix_epoch_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    PerfReport {
        schema_version: 1,
        generated_unix_epoch_sec,
        config,
        scenarios: vec![
            summarize_samples("sprite_renderable_build", &sprite_samples),
            summarize_samples("ui_text_command_build", &ui_samples),
            summarize_samples("scene_construct_destruct", &scene_samples),
        ],
    }
}

fn benchmark_samples<F>(warmup_iterations: u32, iterations: u32, mut func: F) -> Vec<f64>
where
    F: FnMut(),
{
    for _ in 0..warmup_iterations {
        func();
    }

    let mut samples = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let started_at = Instant::now();
        func();
        samples.push(started_at.elapsed().as_secs_f64() * 1000.0);
    }
    samples
}

fn summarize_samples(name: &str, samples: &[f64]) -> PerfScenarioResult {
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let avg_ms = if samples.is_empty() {
        0.0
    } else {
        samples.iter().sum::<f64>() / samples.len() as f64
    };
    let min_ms = sorted.first().copied().unwrap_or(0.0);
    let max_ms = sorted.last().copied().unwrap_or(0.0);
    let p95_ms = if sorted.is_empty() {
        0.0
    } else {
        let index = ((sorted.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
        sorted[index]
    };

    PerfScenarioResult {
        name: name.to_string(),
        avg_ms,
        p95_ms,
        min_ms,
        max_ms,
        iterations: samples.len() as u32,
    }
}

fn build_sprite_world(sprite_count: usize) -> InternalWorld {
    let mut world = InternalWorld::new();
    for i in 0..sprite_count {
        let x = (i % 200) as f32 * 4.0;
        let y = (i / 200) as f32 * 4.0;
        world.spawn((
            ffi::Transform {
                position: ffi::Vec3 { x, y, z: 0.0 },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 1.0,
                },
            },
            Material { texture_handle: 1 },
            Sprite,
        ));
    }
    world
}

fn build_texture_map(sprite_count: usize) -> HashMap<u32, u32> {
    let mut texture_map = HashMap::new();
    // 参照が常に成立する状態を作り、build_renderables 相当の処理だけを計測する。
    texture_map.insert(1, (sprite_count.max(1)) as u32);
    texture_map
}

fn build_renderables_from_world(
    world: &InternalWorld,
    texture_map: &HashMap<u32, u32>,
) -> Vec<ffi::RenderableObject> {
    let mut renderables = Vec::new();
    for archetype in &world.archetypes {
        let has_transform = archetype.types.contains(&ComponentType::Transform);
        let has_material = archetype.types.contains(&ComponentType::Material);
        if !has_transform || !has_material {
            continue;
        }

        let Some(transform_storage) = archetype.storage.get(&ComponentType::Transform) else {
            continue;
        };
        let Some(material_storage) = archetype.storage.get(&ComponentType::Material) else {
            continue;
        };

        let Some(transforms) = transform_storage.downcast_ref::<Vec<ffi::Transform>>() else {
            continue;
        };
        let Some(materials) = material_storage.downcast_ref::<Vec<Material>>() else {
            continue;
        };

        for (transform, material) in transforms.iter().zip(materials.iter()) {
            let texture_id = texture_map
                .get(&material.texture_handle)
                .copied()
                .unwrap_or(0);
            renderables.push(ffi::RenderableObject {
                transform: *transform,
                mesh_id: 1,
                material_id: 1,
                texture_id,
            });
        }
    }
    renderables
}

fn build_ui_text_commands(items_per_row: usize, items_per_col: usize) -> Vec<ffi::TextCommand> {
    let mut text_commands = Vec::with_capacity(items_per_row.saturating_mul(items_per_col));
    let mut count: usize = 0;

    for i in 0..items_per_col {
        for j in 0..items_per_row {
            count += 1;
            text_commands.push(ffi::TextCommand {
                text: format!("T{}", count),
                position: ffi::Vec2 {
                    x: 5.0 + (j as f32 * (800.0 / items_per_row.max(1) as f32)),
                    y: 15.0 + (i as f32 * (600.0 / items_per_col.max(1) as f32)),
                },
                font_size: 12.0,
                color: ffi::Vec4 {
                    x: 0.8,
                    y: 0.8,
                    z: 0.1,
                    w: 1.0,
                },
            });
        }
    }

    text_commands
}

fn run_scene_construct_destruct_cycle(entity_count: usize) -> usize {
    let mut world = InternalWorld::new();
    for i in 0..entity_count {
        let x = (i % 100) as f32 * 8.0;
        let y = (i / 100) as f32 * 8.0;
        world.spawn((
            ffi::Transform {
                position: ffi::Vec3 { x, y, z: 0.0 },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 1.0,
                },
            },
            Material { texture_handle: 1 },
            Sprite,
        ));
    }
    world.clear_entities_of_component(ComponentType::Sprite);
    world.entities.len()
}
