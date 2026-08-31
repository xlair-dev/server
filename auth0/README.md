# Auth0 構成

このディレクトリには、Auth0 Deploy CLI（`a0deploy`）で管理する Auth0 Tenant の構成を置きます。

利用する Tenant は次の 1 つです。

```text
dev-2dn3mvmvr8tccoss.us.auth0.com
```

## ローカルでの実行

Deploy CLI をインストールし、次の環境変数を設定します。

```text
AUTH0_DOMAIN=dev-2dn3mvmvr8tccoss.us.auth0.com
AUTH0_CLIENT_ID=<Deploy CLI 用 client ID>
AUTH0_CLIENT_SECRET=<Deploy CLI 用 client secret>
AUTH0_GITHUB_CLIENT_ID=<GitHub OAuth App の client ID>
AUTH0_GITHUB_CLIENT_SECRET=<GitHub OAuth App の client secret>
XLAIR_GITHUB_APP_CLIENT_ID=<GitHub App の Client ID>
XLAIR_GITHUB_APP_PRIVATE_KEY_BASE64=<GitHub App の private key を base64 化した値>
XLAIR_GITHUB_APP_INSTALLATION_ID=<xlair-dev への installation ID>
```

変更内容の確認:

```sh
a0deploy import \
  --config_file=auth0/config.json \
  --input_file=auth0/tenant.yaml \
  --dry-run
```

変更の適用:

```sh
a0deploy import \
  --config_file=auth0/config.json \
  --input_file=auth0/tenant.yaml \
  --dry-run \
  --apply
```

Deploy CLI 用の Auth0 Management API client は構成管理専用とし、このリポジトリで管理するリソースに必要な権限だけを付与します。credentials は環境変数または CI secrets から渡します。

`auth0/` 以下の変更が `main` に反映されると、GitHub Actions が構成を自動適用します。削除は `AUTH0_ALLOW_DELETE=false` により無効にしています。

## GitHub Actions の初期設定

最初に、Auth0 Dashboard で Deploy CLI 専用の M2M Application を 1 つ作成します。この Application は Auth0 Management API を操作するためのもので、共有 M2M Application（筐体用）とは別です。

Deploy CLI が管理するリソースに必要な Management API の権限を付与し、次の値を GitHub repository の Actions secrets に登録します。

```text
AUTH0_DOMAIN=dev-2dn3mvmvr8tccoss.us.auth0.com
AUTH0_CLIENT_ID=<Deploy CLI 用 client ID>
AUTH0_CLIENT_SECRET=<Deploy CLI 用 client secret>
```

構成には Resource Server、Application、Connection、Action、client grant が含まれるため、Management API には少なくとも次の権限が必要です。

```text
read:resource_servers
create:resource_servers
update:resource_servers
read:clients
create:clients
update:clients

read:client_grants
create:client_grants
update:client_grants

read:connections
create:connections
update:connections

read:actions
create:actions
update:actions

read:triggers
update:triggers
```

削除権限は付与しません。

共有 M2M Application と dashboard 用 Regular Web Application は `tenant.yaml` で管理します。現在はローカル開発用の callback URL、logout URL、web origin を設定しています。dashboard の公開 URL が決まり次第、同じ設定へ追加します。

初回の CI 実行で共有 M2M Application が作成されます。作成後に Auth0 で Client Secret を確認し、秘密情報として筐体へ配布します。Application を事前に手動作成する必要はありません。Client Secret の再発行時は全筐体へ新しい値を再配置します。

GitHub OAuth App（`XLAIR Login`）は事前に作成し、client ID と client secret を GitHub Actions secrets に登録します。Auth0 の GitHub Connection は CI で作成されます。

GitHub OAuth App の Authorization callback URL には次を登録します。

```text
https://dev-2dn3mvmvr8tccoss.us.auth0.com/login/callback
```

GitHub App（例: `XLAIR Auth`）を作成し、`xlair-dev` にインストールします。Organization permissions は `Members: Read-only` だけを付与します。Webhook、Repository permissions、Events は設定しません。

GitHub App の Settings から Client ID と private key を取得し、次の値を GitHub Actions secrets に登録します。

```text
XLAIR_GITHUB_APP_CLIENT_ID=<GitHub App の Client ID>
XLAIR_GITHUB_APP_PRIVATE_KEY_BASE64=<private key の base64 値>
XLAIR_GITHUB_APP_INSTALLATION_ID=<xlair-dev への installation ID>
```

private key は次のように base64 化する。

```sh
base64 -w0 <private-key-file.pem
```

GitHub App は GitHub の設定対象であり、Auth0 Deploy CLI の管理対象ではない。installation token は Action が実行時に発行する。

## 構成モデル

XLAIR API では、筐体の主体を識別する permission として `device` を使用します。

- `device`: 筐体

GitHub の `xlair-dev` membership を確認した Dashboard の user token は、API が Dashboard Application の `azp` によって `Admin` principal に変換します。`device` permission は M2M Application に client grant で付与します。endpoint やフィールド単位の認可は XLAIR 側で扱います。

Deploy CLI 用 Application は構成管理の対象外です。共有 M2M Application と dashboard 用 Regular Web Application は `tenant.yaml` の構成対象です。
