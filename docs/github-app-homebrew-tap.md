# GitHub App For Homebrew Tap Automation

このドキュメントは、`minishelf` の release workflow から `rc-code-jp/homebrew-tap` を自動更新するために、GitHub App を作成して設定する手順をまとめたものです。

公開 OSS でも、この用途では `private GitHub App` で十分です。目的は `minishelf` の GitHub Actions から `homebrew-tap` リポジトリに commit / push することだけなので、不特定ユーザーに配布する必要はありません。

## 前提

- `rc-code-jp/minishelf` と `rc-code-jp/homebrew-tap` が存在していること
- GitHub 上で `rc-code-jp` の設定を変更できること
- `minishelf` の release workflow から `homebrew-tap` を更新したいこと

## 参考にする GitHub 公式ページ

- [Registering a GitHub App](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/registering-a-github-app)
- [Making a GitHub App public or private](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/making-a-github-app-public-or-private)
- [Choosing permissions for a GitHub App](https://docs.github.com/developers/apps/building-github-apps/setting-permissions-for-github-apps)
- [Managing private keys for GitHub Apps](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps)
- [Installing your own GitHub App](https://docs.github.com/apps/installing-github-apps)
- [Making authenticated API requests with a GitHub App in a GitHub Actions workflow](https://docs.github.com/en/enterprise-cloud%40latest/apps/creating-github-apps/authenticating-with-a-github-app/making-authenticated-api-requests-with-a-github-app-in-a-github-actions-workflow)
- [Use GITHUB_TOKEN for authentication in workflows](https://docs.github.com/en/actions/configuring-and-managing-workflows/authenticating-with-the-github_token)

## 1. GitHub App を作成する

GitHub の右上プロフィールから次へ進みます。

`Settings` -> `Developer settings` -> `GitHub Apps` -> `New GitHub App`

入力例:

- `GitHub App name`: `minishelf-homebrew-sync`
- `Description`: `Updates the minishelf Homebrew tap after releases`
- `Homepage URL`: `https://github.com/rc-code-jp/minishelf`

`Webhook` はこの用途では不要なので、`Active` を無効にして構いません。

`Where can this GitHub App be installed?` は `Only on this account` を選びます。

この App は一般配布しないので、`private GitHub App` で問題ありません。

## 2. 権限を最小で設定する

この用途で最初に必要なのは、tap リポジトリへ commit / push するための権限だけです。

Repository permissions:

- `Contents`: `Read and write`

通常はこれで十分です。`Formula/minishelf.rb` を更新するだけなら、`Workflows` 権限は不要です。

## 3. App を作成する

ページ下部の `Create GitHub App` を押して作成します。

作成後、App の設定画面で `App ID` を控えてください。後で GitHub Actions の variable に使います。

## 4. Private key を生成する

App 設定画面の `Private keys` セクションで `Generate a private key` を押します。

`.pem` ファイルはその場でダウンロードされます。GitHub 上からあとで同じ内容を再表示できないので、安全な場所に保管してください。

## 5. App をインストールする

App 設定画面の `Install App` から `rc-code-jp` にインストールします。

Repository access は `Only select repositories` を選び、少なくとも次の 2 つを追加します。

- `rc-code-jp/minishelf`
- `rc-code-jp/homebrew-tap`

`minishelf` 側で workflow を実行し、その token で `homebrew-tap` を更新するため、この 2 リポジトリに入れておくのが扱いやすい構成です。

## 6. `minishelf` に Actions secrets / variables を追加する

`rc-code-jp/minishelf` の次の画面を開きます。

`Settings` -> `Secrets and variables` -> `Actions`

追加するもの:

- Repository variable: `APP_ID`
- Repository secret: `APP_PRIVATE_KEY`

値:

- `APP_ID`
  - GitHub App の設定画面に表示される `App ID`
- `APP_PRIVATE_KEY`
  - ダウンロードした `.pem` ファイルの全文
  - `-----BEGIN ...-----` と `-----END ...-----` を含めてそのまま貼る

## 7. Workflow で installation token を発行する

GitHub 公式のやり方に沿うなら、Actions では `actions/create-github-app-token` を使います。

例:

```yaml
- name: Generate GitHub App token
  id: app-token
  uses: actions/create-github-app-token@v2
  with:
    app-id: ${{ vars.APP_ID }}
    private-key: ${{ secrets.APP_PRIVATE_KEY }}
    owner: rc-code-jp
```

この token を使って `homebrew-tap` を checkout します。

```yaml
- name: Checkout tap repo
  uses: actions/checkout@v4
  with:
    repository: rc-code-jp/homebrew-tap
    token: ${{ steps.app-token.outputs.token }}
    path: homebrew-tap
```

その後に `Formula/minishelf.rb` の `version` と `sha256` を更新して commit / push します。

## 8. なぜ `GITHUB_TOKEN` ではなく GitHub App なのか

`GITHUB_TOKEN` は基本的に workflow が動いている同じリポジトリ向けです。今回のように `minishelf` から別リポジトリの `homebrew-tap` へ安全に書き込みたい場合は、GitHub 公式の案内どおり GitHub App を使うのが自然です。

fine-grained PAT でも実現はできますが、長期運用する OSS では次の点で GitHub App のほうが適しています。

- リポジトリ単位で対象を絞りやすい
- 権限を細かく制限しやすい
- 個人アカウント依存を減らせる
- 将来メンテナーが増えても管理しやすい

## 9. 導入後にやること

GitHub App の設定が終わったら、`minishelf` 側で次を実装します。

- release workflow で GitHub App token を発行する
- `rc-code-jp/homebrew-tap` を checkout する
- `Formula/minishelf.rb` の `version` と `sha256` を更新する
- `git commit` して `git push` する

必要なら、次のステップとして `.github/workflows/release.yml` に tap 自動更新ジョブを追加してください。
