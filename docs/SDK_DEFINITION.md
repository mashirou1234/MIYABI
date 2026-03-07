# MIYABI SDK定義 (v0.1)

最終更新: 2026-03-06

## 1. 目的

MIYABI SDK v0.1 は、外部 C++ アプリケーションから MIYABI ロジックを静的リンクで利用するための配布物を定義する。

本SDKは「外部ゲーム開発向け」の最小成立ラインとして、以下を提供する。

- ロジックAPI (`MiyabiVTable`) の公開
- ロジックが必要とするランタイムサービス（音声/物理）の提供
- CMakeテンプレートと最小サンプル

## 2. 想定利用形態

- 利用者は C++17 のホストアプリを実装する
- ホストは SDK 同梱の静的ライブラリ群をリンクする
- ロジック呼び出しは `get_miyabi_vtable()` を起点に行う
- `dlopen/dlsym` による動的ホットリロードは v0.1 のスコープ外

## 3. 公開エントリポイント

### 3.1 ロジックAPI起点

- 宣言: `extern "C" MiyabiVTable get_miyabi_vtable();`
- 定義元: `libmiyabi_logic.a`
- 役割: ゲームロジック更新/描画データ取得/アセット要求などを関数テーブルで提供

### 3.2 ランタイムサービス起点

- 宣言: `void init_engine_systems();`, `void step_engine_systems();`, `void shutdown_engine_systems();`
- 定義元: `libmiyabi_runtime.a`
- 役割: 音声・物理など、ロジック側が要求する C++ サービスを初期化/更新

### 3.3 ABI互換判定定数

- 宣言: `MIYABI_ABI_VERSION_MAJOR`, `MIYABI_ABI_VERSION_MINOR`, `MIYABI_ABI_VERSION_PATCH`, `MIYABI_ABI_VERSION`
- 配置: `include/miyabi/miyabi.h`
- 役割: `get_miyabi_vtable()` の戻り値 `MiyabiVTable::abi_version` と比較し、ホストとロジックのABI整合性を実行時に判定する。

## 4. 配布物定義

SDK ZIP (`MIYABI_SDK.zip`) には最低限、以下を含める。

- `include/miyabi/miyabi.h`
- `include/miyabi/bridge.h`
- `include/miyabi_logic_cxx/lib.h`
- `include/rust/cxx.h`
- `lib/libmiyabi_logic.a`
- `lib/libmiyabi_logic_cxx.a`
- `lib/libmiyabi_runtime.a`
- `lib/libbox2d.a`
- `cmake/MIYABIConfig.cmake`
- `cmake/MIYABIConfigVersion.cmake`
- `template_CMakeLists.txt`
- `examples/main.cpp`
- `docs/SDK_DEFINITION.md`

照合対象の実行時チェックは `scripts/check_sdk_artifacts.sh` を正とする。
このスクリプトは上記必須同梱物の欠落を検知し、`sdk/` 以外を検査する場合はディレクトリを引数で指定する。

例:

```bash
./scripts/check_sdk_artifacts.sh
./scripts/check_sdk_artifacts.sh --dry-run
./scripts/check_sdk_artifacts.sh /path/to/extracted/sdk
```

補助検証として Python チェッカーも利用できる。

```bash
python3 tools/check_sdk_bundle.py --sdk-dir ./sdk
```

不足がある場合は非0で終了し、不足項目一覧を表示する。
`--dry-run` 指定時は不足項目を表示しつつ終了コード 0 で返す。

### 4.1 SDK更新時チェックリスト（`build_sdk.sh` 変更時）

`build_sdk.sh` を更新した場合は、配布物定義と実装の同期漏れを防ぐために、以下を必ず確認する。

1. 配布物一覧の同期
   - `build_sdk.sh` の `REQUIRED_ARTIFACTS` と本書「4. 配布物定義」の列挙が一致していること。
2. コピー処理の同期
   - 追加/削除した配布物に対応する `cp` / `mkdir -p` / `find` の処理が `build_sdk.sh` に存在すること。
3. CMakeパッケージ同梱の同期
   - `cmake/MIYABIConfig.cmake` と `cmake/MIYABIConfigVersion.cmake` の同梱処理が維持されていること。
4. サンプルとテンプレート同梱の同期
   - `examples/main.cpp` と `template_CMakeLists.txt` が継続して同梱されること。
5. 定義書同梱の同期
   - `docs/SDK_DEFINITION.md` 自体が SDK に同梱され、配布物定義の正として参照可能なこと。
6. 同梱物検証の実行
   - `./scripts/check_sdk_artifacts.sh <sdk_dir>` が成功し、欠落項目が 0 件であること。

### 4.2 ABI確認の最短手順

SDK展開後、最短で ABI 整合を確認する手順を以下に固定する。

1. 同梱漏れの一次診断（非破壊）
   - `./scripts/check_sdk_artifacts.sh --dry-run <sdk_dir>`
2. 同梱漏れの厳密検証（失敗時は非0）
   - `./scripts/check_sdk_artifacts.sh <sdk_dir>`
3. 実行時 ABI 判定の確認
   - `sdk/examples/main.cpp` 相当で `vtable.abi_version == MIYABI_ABI_VERSION` を実行し、判定が true であることを確認する。

### 4.3 `check_sdk_artifacts` JSON出力オプション設計メモ（運用差分）

`scripts/check_sdk_artifacts.sh` は現状テキスト出力のみを提供する。  
結果共有の転記コストを下げるため、次回拡張で以下の JSON 出力オプションを追加する設計を採用する。

- 対象コマンド
  - `./scripts/check_sdk_artifacts.sh --json`
  - `./scripts/check_sdk_artifacts.sh --json=/path/to/report.json`
- 非互換回避方針
  - `--json` 未指定時の既存標準出力/標準エラー文言と終了コードは変更しない。
  - `--dry-run` の意味（不足があっても終了コード 0）は維持する。
- 最小 JSON スキーマ（案）
  - `schema_version`: 文字列。初期値は `"1"`.
  - `sdk_dir`: 検査対象ディレクトリ。
  - `dry_run`: 真偽値。
  - `ok`: 真偽値。必須同梱物が全件存在する場合のみ `true`。
  - `missing`: 不足ファイル相対パスの配列。
  - `required_count`: 必須件数。
  - `checked_at`: UTC ISO8601 文字列。
- 運用手順（15〜45 分で実施）
  1. `scripts/check_sdk_artifacts.sh` に `--json` パースを追加する。
  2. 既存の欠落判定ロジックから `missing` 配列を組み立てる。
  3. `jq -n` で JSON を生成し、`--json=<path>` 指定時のみファイル出力する。
  4. `--json` 指定時でも終了コード判定は既存仕様（`--dry-run` 含む）に合わせる。
  5. `README.md` と本節を参照して運用者へ共有する。

補足:

- 既存 #179 / #178 は `build_sdk.sh` の前提チェックが対象であり、本設計メモは `check_sdk_artifacts` の出力形式拡張に限定する。
## 5. リンク契約

`find_package(MIYABI CONFIG REQUIRED)` により `MIYABI::SDK` を利用する。
`MIYABI::SDK` は内部で以下の順序でリンクされる。

1. `MIYABI::miyabi_logic`
2. `MIYABI::miyabi_logic_cxx`
3. `MIYABI::miyabi_runtime`
4. `MIYABI::box2d`

推奨テンプレートは `sdk/template_CMakeLists.txt` を正とする。

## 6. 実行契約

最小実行フロー:

1. `init_engine_systems()`
2. `get_miyabi_vtable()` + `vtable.abi_version == MIYABI_ABI_VERSION` を検証
3. `create_game()`
4. 毎フレーム `step_engine_systems()` → `update_game()`
5. 終了時 `destroy_game()`
6. 終了時 `shutdown_engine_systems()`

所有権/メモリルールは `docs/DESIGN_FFI.md` と `core/include/miyabi/miyabi.h` を正とする。

## 7. 非スコープ (v0.1)

- ABI互換性の長期保証（同一ZIP内整合のみ保証）
- Windows/Linux の動作保証
- 動的ロジック差し替え (`dlopen/dlsym`) の公式サポート

## 8. 今後の拡張方針

- ABI更新時の移行ポリシー（互換/非互換判定）の明文化
- プラットフォーム別の公式配布とCIによる検証

## 9. ABI更新ポリシー

`MIYABI_ABI_VERSION = (major << 16) | (minor << 8) | patch` とする。

- `major`:
  - `MiyabiVTable` レイアウト変更、関数削除、引数型変更などの**非互換変更**でインクリメントする。
  - 変更時は、既存SDK利用側コードの修正が必須になる。
- `minor`:
  - 既存契約を壊さない関数追加などの**後方互換変更**でインクリメントする。
  - 既存利用側は再ビルドのみで動作継続できる前提とする。
- `patch`:
  - 不具合修正・内部実装変更など、ABIに影響しない変更でインクリメントする。

運用ルール:

1. ABIに関わるPRでは `core/include/miyabi/miyabi.h` のバージョン定数更新有無をレビュー項目に含める。
2. SDK配布時は `sdk/examples/main.cpp` で `vtable.abi_version` の比較を維持する。
3. CIでは `.github/workflows/build.yml` の `SDK ABI smoke` ステップで `sdk_template_main.cpp` 相当の `vtable.abi_version == MIYABI_ABI_VERSION` 判定を実行し、失敗時はジョブを fail させる。
4. `major` 変更時は `docs/CODEX_MIGRATION_STATUS.md` に移行手順（影響範囲/変更点）を明記する。

### 9.1 破壊的変更（major更新）時のバージョニング規則

`major` を更新する変更は、次の 3 条件をすべて満たした場合にのみリリース可能とする。

1. 破壊的変更の根拠を明示
   - 変更PRで、対象 API と非互換理由（構造体レイアウト変更 / 関数削除 / 引数型変更など）を明記する。
2. バージョン値を正規化
   - `core/include/miyabi/miyabi.h` の ABI 定数で `major` を +1 し、`minor` と `patch` を 0 に戻す。
3. 移行情報を同時公開
   - `docs/CODEX_MIGRATION_STATUS.md` に「影響範囲」「利用者側の必要修正」「検証手順」を追記する。

補足ルール:

- 破壊的変更を含む run では、`PLAN.md` の対象タスクに ABI 変更有無を追記し、後続 run が判断できる状態を維持する。
- `major` 更新を伴う差分は、非破壊変更（`minor` / `patch`）と同一PRに混在させない。
