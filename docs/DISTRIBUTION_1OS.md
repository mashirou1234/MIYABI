# MIYABI 1OS配布手順（macOS）

最終更新: 2026-02-23

## 1. 目的

`PLAN.md` タスク9.3「配布可能ビルドの成立（1OS）」に対し、macOS 向け配布ビルド手順を固定する。

## 2. 前提

- 対象OS: macOS
- 必要ツール:
  - `cmake`
  - C++17 コンパイラ（AppleClang）
  - Rust stable
  - `zip`

## 3. 配布パッケージ生成

リポジトリルートで次を実行する。

```bash
./scripts/package_macos_game.sh
```

実行結果:

- `dist/MIYABI_GAME_macOS_<timestamp>.zip` を生成
- 展開用ディレクトリ `dist/miyabi_game_macos` を生成
  - `bin/miyabi`
  - `assets/*`
  - `run_miyabi.sh`
  - `SHA256SUMS.txt`
  - `docs/README.txt`

## 4. クリーン再現確認（同一OS）

1. `build_release_game` が都度削除されることを確認する（クリーンビルド）
2. 生成 ZIP を別ディレクトリへ展開する
3. 展開先で次を実行する

```bash
./run_miyabi.sh
```

4. タイトル画面が表示され、`Start` から `InGame` へ遷移できることを確認する

## 5. 既知制約

- 署名（codesign）/ notarization は未対応。
- 本手順は「1OSでの再現可能配布手順固定」を対象とし、マルチOS配布は対象外。
