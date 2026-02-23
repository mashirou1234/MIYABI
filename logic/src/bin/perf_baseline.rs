use miyabi_logic::perf::{run_performance_baseline, PerfConfig};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn parse_u32_arg(flag: &str, value: Option<String>) -> Result<u32, Box<dyn Error>> {
    let value = value.ok_or_else(|| format!("{flag} には数値が必要です"))?;
    Ok(value.parse::<u32>()?)
}

fn parse_usize_arg(flag: &str, value: Option<String>) -> Result<usize, Box<dyn Error>> {
    let value = value.ok_or_else(|| format!("{flag} には数値が必要です"))?;
    Ok(value.parse::<usize>()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = PerfConfig::default();
    let mut output_path = PathBuf::from("build/perf/current_baseline.json");

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output_path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--output には出力パスが必要です".to_string())?,
                );
            }
            "--iterations" => {
                config.iterations = parse_u32_arg("--iterations", args.next())?;
            }
            "--warmup" => {
                config.warmup_iterations = parse_u32_arg("--warmup", args.next())?;
            }
            "--sprite-count" => {
                config.sprite_count = parse_usize_arg("--sprite-count", args.next())?;
            }
            "--ui-rows" => {
                config.ui_items_per_row = parse_usize_arg("--ui-rows", args.next())?;
            }
            "--ui-cols" => {
                config.ui_items_per_col = parse_usize_arg("--ui-cols", args.next())?;
            }
            "--scene-entities" => {
                config.scene_entity_count = parse_usize_arg("--scene-entities", args.next())?;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                return Err(format!("未対応オプションです: {arg}").into());
            }
        }
    }

    let report = run_performance_baseline(config);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&output_path, json)?;

    println!("[perf] report={}", output_path.display());
    for scenario in &report.scenarios {
        println!(
            "[perf] scenario={} avg_ms={:.3} p95_ms={:.3} min_ms={:.3} max_ms={:.3} iterations={}",
            scenario.name,
            scenario.avg_ms,
            scenario.p95_ms,
            scenario.min_ms,
            scenario.max_ms,
            scenario.iterations
        );
    }

    Ok(())
}

fn print_help() {
    println!("Usage: perf_baseline [options]");
    println!("  --output <path>         JSON出力先 (default: build/perf/current_baseline.json)");
    println!("  --iterations <n>        計測反復数 (default: 30)");
    println!("  --warmup <n>            ウォームアップ反復数 (default: 5)");
    println!("  --sprite-count <n>      スプライト数 (default: 10000)");
    println!("  --ui-rows <n>           UI行数 (default: 30)");
    println!("  --ui-cols <n>           UI列数 (default: 40)");
    println!("  --scene-entities <n>    シーン構築破棄のエンティティ数 (default: 5000)");
}
