use crate::{ffi, ComponentType, InternalWorld, Material, Sprite};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const PERF_SCENARIO_SPRITE_RENDERABLE_BUILD: &str = "sprite_renderable_build";
pub const PERF_SCENARIO_UI_TEXT_COMMAND_BUILD: &str = "ui_text_command_build";
pub const PERF_SCENARIO_SCENE_CONSTRUCT_DESTRUCT: &str = "scene_construct_destruct";

pub const PERF_SCENARIO_KEYS: [&str; 3] = [
    PERF_SCENARIO_SPRITE_RENDERABLE_BUILD,
    PERF_SCENARIO_UI_TEXT_COMMAND_BUILD,
    PERF_SCENARIO_SCENE_CONSTRUCT_DESTRUCT,
];

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
    pub git_commit: String,
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
        let _text_commands =
            build_ui_text_commands(config.ui_items_per_row, config.ui_items_per_col);
    });

    let scene_samples = benchmark_samples(config.warmup_iterations, config.iterations, || {
        let _ = run_scene_construct_destruct_cycle(config.scene_entity_count);
    });

    let generated_unix_epoch_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let git_commit = resolve_git_commit();

    PerfReport {
        schema_version: 1,
        generated_unix_epoch_sec,
        git_commit,
        config,
        scenarios: vec![
            summarize_samples(PERF_SCENARIO_SPRITE_RENDERABLE_BUILD, &sprite_samples),
            summarize_samples(PERF_SCENARIO_UI_TEXT_COMMAND_BUILD, &ui_samples),
            summarize_samples(PERF_SCENARIO_SCENE_CONSTRUCT_DESTRUCT, &scene_samples),
        ],
    }
}

fn resolve_git_commit() -> String {
    let github_sha = std::env::var("GITHUB_SHA").ok();
    let git_short_head = resolve_git_short_head();
    choose_git_commit(github_sha.as_deref(), git_short_head.as_deref())
}

fn resolve_git_short_head() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn choose_git_commit(github_sha: Option<&str>, git_short_head: Option<&str>) -> String {
    let github_sha = github_sha.map(str::trim).filter(|value| !value.is_empty());
    let git_short_head = git_short_head
        .map(str::trim)
        .filter(|value| !value.is_empty());
    github_sha
        .or(git_short_head)
        .unwrap_or("unknown")
        .to_string()
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
                is_3d: false,
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

#[cfg(test)]
mod baseline_key_tests {
    use super::PERF_SCENARIO_KEYS;
    use serde_json::Value;
    use std::collections::BTreeSet;

    fn collect_key_diff(
        expected_keys: &BTreeSet<String>,
        actual_keys: &BTreeSet<String>,
    ) -> (Vec<String>, Vec<String>) {
        let missing: Vec<String> = expected_keys.difference(actual_keys).cloned().collect();
        let extra: Vec<String> = actual_keys.difference(expected_keys).cloned().collect();
        (missing, extra)
    }

    #[test]
    fn baseline_scenario_keys_match_perf_report_keys() {
        let baseline_raw = include_str!("../../docs/perf/baseline_macos14.json");
        let baseline: Value =
            serde_json::from_str(baseline_raw).expect("baseline_macos14.json must be valid JSON");

        let baseline_scenarios = baseline
            .get("scenarios")
            .and_then(Value::as_array)
            .expect("baseline_macos14.json must have scenarios array");

        let mut baseline_keys = BTreeSet::new();
        for scenario in baseline_scenarios {
            let scenario_name = scenario
                .get("name")
                .and_then(Value::as_str)
                .expect("each scenario must have name");
            baseline_keys.insert(scenario_name.to_string());
        }

        let expected_keys: BTreeSet<String> =
            PERF_SCENARIO_KEYS.iter().map(|item| item.to_string()).collect();
        let (missing, extra) = collect_key_diff(&expected_keys, &baseline_keys);
        assert!(
            missing.is_empty() && extra.is_empty(),
            "perf baseline keys mismatch: missing={missing:?} extra={extra:?}"
        );
    }

    #[test]
    fn detect_missing_and_extra_scenario_keys() {
        let expected_keys: BTreeSet<String> =
            PERF_SCENARIO_KEYS.iter().map(|item| item.to_string()).collect();
        let actual_keys = BTreeSet::from([
            "sprite_renderable_build".to_string(),
            "unknown_extra_key".to_string(),
        ]);
        let (missing, extra) = collect_key_diff(&expected_keys, &actual_keys);
        assert_eq!(
            missing,
            vec![
                "scene_construct_destruct".to_string(),
                "ui_text_command_build".to_string()
            ]
        );
        assert_eq!(extra, vec!["unknown_extra_key".to_string()]);
    }
}

#[cfg(test)]
mod tests {
    use super::choose_git_commit;

    #[test]
    fn choose_git_commit_prefers_github_sha() {
        let commit = choose_git_commit(Some("abcdef123456"), Some("deadbee"));
        assert_eq!(commit, "abcdef123456");
    }

    #[test]
    fn choose_git_commit_uses_git_when_github_sha_missing() {
        let commit = choose_git_commit(None, Some("deadbee"));
        assert_eq!(commit, "deadbee");
    }

    #[test]
    fn choose_git_commit_falls_back_to_unknown() {
        let commit = choose_git_commit(Some("   "), Some(""));
        assert_eq!(commit, "unknown");
    }
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
