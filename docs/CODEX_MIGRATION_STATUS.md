# Codex移行ステータス

最終更新: 2026-02-23

## 1. 現在の到達点

- `cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON`
- `cmake --build build -j4`

上記コマンドで `build/core/miyabi` まで生成できる状態に復旧済み。

## 2. この移行で反映した内容

- cxx生成ヘッダ参照を `miyabi_logic_cxx/lib.h` に統一
  - `core/include/miyabi/miyabi.h`
  - `core/src/miyabi_bridge.cpp`
  - `core/src/physics/PhysicsManager.cpp`
- `core` から `miyabi_logic_cxx` の直接リンクを外し、重複シンボルを回避
  - `core/CMakeLists.txt`
- Rust警告の解消（`Box::from_raw` / `CString::from_raw` の戻り値処理など）
  - `logic/src/lib.rs`
  - `logic/src/paths.rs.in`
- パフォーマンステスト文書の進捗と typo を更新
  - `PERFORMANCE_TEST.md`
- SDK生成スクリプトを現行出力に合わせて修正
  - `build_sdk.sh`
  - `sdk_template_CMakeLists.txt`
- 設計ドキュメントに「現行は静的リンク」注記を追加
  - `docs/DESIGN_Build.md`
  - `docs/DESIGN_FFI.md`

## 3. 現在の構成（正）

- Rustロジック: `logic` クレート（`staticlib`）
- cxxブリッジ生成物: `miyabi_logic_cxx` ターゲット
- C++ホスト: `core` の `miyabi` 実行ファイル
- 呼び出し契約: `get_miyabi_vtable()` を静的リンクして利用

## 4. 残課題（次スレッド優先）

1. SDKの定義を明確化
   - 現在のSDKは「ランタイム + logic静的ライブラリ」寄り。
   - 「外部ゲーム開発向けSDK」として成立させるには、公開API/エントリポイント設計の再定義が必要。
2. CI導入
   - `.github/workflows` が未整備。
   - まずは configure/build の自動実行を追加する。
3. リンカ警告の整理
   - duplicate libraries warning
   - macOS deployment target warning（26.2 vs 26.0）
4. 性能計画の未完了タスク
   - シーン構築/破棄ストレステスト
   - ベースライン記録

## 5. 続スレッド再開コマンド

```bash
cd /Users/shiroguchi/Documents/Github/mashirou1234/Game/MIYABI
git status --short --branch
cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON
cmake --build build -j4
```

