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

構成には Resource Server、Application、Role、client grant が含まれるため、Management API には少なくとも次の権限が必要です。

```text
read:resource_servers
create:resource_servers
update:resource_servers
read:roles
create:roles
update:roles

read:clients
create:clients
update:clients

read:client_grants
create:client_grants
update:client_grants
```

削除権限は付与しません。

共有 M2M Application と dashboard 用 Regular Web Application は `tenant.yaml` で管理します。dashboard の callback URL、logout URL、web origin は dashboard の公開 URL が決まり次第設定します。

初回の CI 実行で共有 M2M Application が作成されます。作成後に Auth0 で Client Secret を確認し、秘密情報として筐体へ配布します。Application を事前に手動作成する必要はありません。Client Secret の再発行時は全筐体へ新しい値を再配置します。

## 構成モデル

XLAIR API では、主体の大分類として次の permission を使用します。

- `admin`: 管理者
- `device`: 筐体

`admin` role には `admin` permission を設定します。`device` permission は共有 M2M Application に client grant で付与します。endpoint やフィールド単位の認可は XLAIR 側で扱います。

Deploy CLI 用 Application は構成管理の対象外です。共有 M2M Application と dashboard 用 Regular Web Application は `tenant.yaml` の構成対象です。
