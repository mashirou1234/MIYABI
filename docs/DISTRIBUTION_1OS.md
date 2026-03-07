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

事前診断のみを行う場合:

```bash
./scripts/package_macos_game.sh --preflight-only
```

`--preflight-only` では依存コマンド、必須アセット、書き込み可能性を検証し、ビルドとZIP生成は行わない。

実行結果:

- `dist/MIYABI_GAME_macOS_<timestamp>.zip` を生成
- 展開用ディレクトリ `dist/miyabi_game_macos` を生成
  - `bin/miyabi`
  - `assets/*`
  - `run_miyabi.sh`
  - `SHA256SUMS.txt`
  - `docs/README.txt`

### 3.1 最低同梱物チェック

配布前に、展開ディレクトリ `dist/miyabi_game_macos` に次の必須同梱物があることを確認する。

- `bin/miyabi`
- `assets/player.png`
- `assets/test.png`
- `assets/test_sound.wav`
- `shaders/text.vert`
- `shaders/text.frag`
- `shaders/textured.vert`
- `shaders/textured.frag`
- `run_miyabi.sh`
- `SHA256SUMS.txt`
- `docs/README.txt`

確認コマンド例:

```bash
for p in \
  bin/miyabi \
  assets/player.png \
  assets/test.png \
  assets/test_sound.wav \
  shaders/text.vert \
  shaders/text.frag \
  shaders/textured.vert \
  shaders/textured.frag \
  run_miyabi.sh \
  SHA256SUMS.txt \
  docs/README.txt; do
  test -e "dist/miyabi_game_macos/$p" || { echo "missing: $p"; exit 1; }
done
```

### 3.2 成果物署名確認（手動署名時）

`scripts/package_macos_game.sh` は署名付与を行わないため、配布担当が手動署名した成果物のみ本手順で確認する。

前提:

- 検証対象: `dist/miyabi_game_macos/bin/miyabi`
- 期待 Team ID（例）: `TEAMID1234`（配布チャンネルで固定した値を使用）

確認手順:

```bash
codesign --verify --deep --strict --verbose=2 dist/miyabi_game_macos/bin/miyabi
codesign -dv --verbose=4 dist/miyabi_game_macos/bin/miyabi 2>&1 | rg 'TeamIdentifier|Authority'
spctl --assess --type execute --verbose=4 dist/miyabi_game_macos/bin/miyabi
```

判定条件:

- `codesign --verify` が `valid on disk` と `satisfies its Designated Requirement` を返す
- `codesign -dv` の `TeamIdentifier` がリリース計画で合意した値と一致する
- `spctl --assess` が `accepted` を返す

補足:

- 未署名成果物を配布する場合は、`docs/README.txt` に「未署名配布」である旨を明記する。
- 署名運用の範囲は [PLAN.md](../PLAN.md) の配布タスクに合わせて更新する。

## 4. クリーン再現確認（同一OS）

1. `build_release_game` が都度削除されることを確認する（クリーンビルド）
2. 生成 ZIP を別ディレクトリへ展開する
3. 展開先で次を実行する

```bash
./run_miyabi.sh
```

4. タイトル画面が表示され、`Start` から `InGame` へ遷移できることを確認する

### 4.1 再現スモーク自動化

同一手順の再実行を優先する場合は、次のコマンドを使う。

```bash
./scripts/test_distribution_smoke.sh
```

このスクリプトは次をまとめて行う。

- 最新の配布 ZIP を生成する
- 一時ディレクトリへ展開する
- 最低同梱物を検証する
- 展開先の `./run_miyabi.sh` を 5 秒間起動し、即時クラッシュしないことを確認する

Issue コメントまたは PR には、出力された ZIP パスと PASS/FAIL をそのまま転記する。

## 5. 既知制約

- `scripts/package_macos_game.sh` は 署名（codesign）/ notarization を自動実行しない。
- 本手順は「1OSでの再現可能配布手順固定」を対象とし、マルチOS配布は対象外。
