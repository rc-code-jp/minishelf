# Release Guide

このドキュメントは `minishelf` のメンテナー向けリリース手順です。

## Prerequisites

- `main` ブランチが最新であること
- ワークツリーが clean であること
- ローカルで次の確認が通ること
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -D warnings`
  - `cargo test`

## 1. リリースタグを作成して push

バージョン更新を commit した後、タグを push します。

```bash
git tag v0.1.15
git push origin v0.1.15
```

`push` されたタグで `.github/workflows/release.yml` が起動し、以下を公開します。

- `minishelf-<version>-linux-x86_64.tar.gz`
- `minishelf-<version>-macos-aarch64.tar.gz`
- `checksums.txt`
- `checksums.txt.sig`
- `checksums.txt.pem`

## 2. 手動実行する場合（workflow_dispatch）

GitHub Actions の `release` workflow を手動起動する場合は、`tag` 入力に `vX.Y.Z` 形式の既存タグを指定してください（例: `v0.1.15`）。

- `tag` は必須です
- `v` で始まらない値は失敗します

## 3. Homebrew Formula 更新PRを作成

同ワークフローの `update_formula` ジョブが `chore/formula-<tag>` ブランチを push します。
Actions Summary に表示される compare URL から PR を作成し、レビュー後にマージしてください。

- `Formula/minishelf.rb` は `main` へ直接 push しない
- 変更は PR 経由で反映する

## 4. checksums 署名の検証（cosign keyless）

`checksums.txt` の署名は keyless で発行します。ローカルでの検証例:

```bash
cosign verify-blob \
  --certificate checksums.txt.pem \
  --signature checksums.txt.sig \
  --certificate-identity-regexp 'https://github.com/rc-code-jp/minishelf/.github/workflows/release.yml@.*' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  checksums.txt
```

`Verified OK` を確認してください。

## Troubleshooting

- `Missing checksum(s) for version ...`
  - タグとアーティファクト名のバージョンが一致しているか確認する
- `workflow_dispatch` で失敗する
  - `tag` に `vX.Y.Z` 形式の既存タグを指定しているか確認する
