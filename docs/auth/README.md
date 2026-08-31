# 認証・認可設計

利用手順は [筐体の認証](./device.md) と [管理 dashboard の認証](./dashboard.md) を参照する。

## 決定事項

| 主体 | 認証 | XLAIR 上の識別 |
| --- | --- | --- |
| 一般ユーザー | 筐体で読み取った card ID | `users.card` |
| 筐体 | Auth0 M2M access token | Auth0 の client identity |
| 管理者・運用者 | GitHub OAuth を経由した Auth0 user access token | Auth0 の user identity |

- 一般ユーザーは Auth0 にアカウントを作成しない
- card ID を XLAIR 上のユーザー識別子として扱う
- カードの複製による同一ユーザーとしての利用を許容する
- 筐体認証には Auth0 M2M を使用する
- 全筐体で 1 つの M2M Application と credentials を共有する
- 筐体 credentials に定期的な有効期限は設けない
- 管理者・運用者の認証には GitHub OAuth と Auth0 を使用する
- OpenAPI では `userAuth` と `deviceAuth` を使用する
- `/users/{userId}` は筐体と管理者の双方から更新できる
- 更新メソッドは `PATCH` とする

## Auth0 と XLAIR の責務

GitHub は管理者・運用者の identity と組織所属を提供し、Auth0 は認証フローと token 発行を担当する。XLAIR の endpoint、リソース、フィールド単位の認可は XLAIR 側で管理する。

```text
Auth0
  └─ GitHub login の仲介、token 発行、管理者・筐体の識別

GitHub
  └─ 管理者・運用者の identity、xlair-dev の membership

XLAIR
  └─ endpoint、リソース、操作内容に対する認可
```

`admin` と `device` は細かな API permission ではなく、XLAIR が認可ポリシーを選択するための principal の種類として扱う。

採用する Auth0 上の表現は次のとおりとする。管理者は GitHub の `xlair-dev` membership を GitHub App を使う Post-Login Action で確認し、API は Dashboard Application の `azp` を `Admin` principal に変換する。

```text
Dashboard Application
  └─ GitHub identity を認証した user token

device M2M client grant
  └─ device permission
```

Auth0 には endpoint やフィールド単位の permission を登録しない。`device` permission は M2M client の主体識別にだけ使用し、管理者の主体識別には Dashboard Application の `azp` を使用する。これにより、XLAIR の認可モデルを Auth0 の Role / Permission に依存させない。

API 層は token の `permissions`、`azp`、token の主体種別を検証し、XLAIR 内部の principal に変換する。`scope` は OAuth の token 上の表現として扱い、アプリケーションの認可モデルには持ち込まない。custom claim は使用しない。

```text
DevicePrincipal { client_id }
UserPrincipal { subject }
```

`device` permission を持つ client credentials token は `DevicePrincipal` に、Dashboard Application の `azp` を持つ user token は `Admin` principal に変換する。GitHub の `xlair-dev` membership は GitHub App の installation token を使う Auth0 Post-Login Action で確認する。以降の endpoint・リソース・フィールド単位の認可は XLAIR 側で行う。

現在の endpoint 認可では、private route に `device` を要求する。`/health`、`/rankings`、`/statistics/summary` は公開する。`admin` principal の endpoint 利用は今後追加する。

## 認証フロー

API サーバーは次の設定で Auth0 access token を検証する。

```text
AUTH0_ISSUER=https://dev-2dn3mvmvr8tccoss.us.auth0.com/
AUTH0_AUDIENCE=https://api.xlair.dev
AUTH0_DASHBOARD_CLIENT_ID=<XLAIR Dashboard の Client ID>
```

`/health`、`/rankings`、`/statistics/summary` 以外の route では Bearer token を必須とする。

### 一般ユーザー

1. 筐体が card ID を読み取る
2. 筐体が `deviceAuth` で API にアクセスする
3. card ID を API の入力として送信する
4. XLAIR が card ID からユーザーを解決する

### 筐体

筐体は共有する Auth0 M2M Application として API 用 access token を取得し、次の形式で送信する。

```http
Authorization: Bearer <device access token>
```

### 管理者・運用者

GitHub OAuth を経由して取得した Auth0 の user access token を `userAuth` として送信する。現時点では管理者・運用者用だが、将来の認証対象拡張を考慮して scheme 名は `userAuth` とする。

各 endpoint の認証方式は [openapi.yaml](../openapi.yaml) に定義する。筐体と管理者の双方が利用する操作は、認証方式を OR で指定する。

## 筐体 credentials の運用（推奨）

筐体の登録時に provisioning 処理を実行し、共有する Auth0 M2M Application の credential を設置担当者または筐体へ安全に渡す。XLAIR の公開 API に credential 発行機能は設けない。

client credential の定期的な失効は行わない。access token は短寿命に設定し、紛失・侵害時は Auth0 で対象 client を無効化するか、credential を rotation する。Auth0 で client を無効化しても発行済み token は有効期限まで有効なため、短寿命 token とする。

credential の発行は、共有 M2M Application を初期構成する provisioning とする。初回の CI 実行で Application と API grant を作成し、生成された Client Secret を Auth0 から取得して筐体へ配布する。公開 API からの自己登録は行わない。

Client Secret を secure な provisioning 経路で筐体へ配布する。

## Auth0 の構成管理

Auth0 の構成管理には Auth0 Deploy CLI（`a0deploy`）を使用する。Tenant の設定をリポジトリで管理し、import 前に dry run で変更内容を確認する。無料プランの制約に合わせ、Tenant は分離しない。

管理対象:

- XLAIR API の Resource Server
- dashboard 用 Regular Web Application
- 筐体で共有する M2M Application
- `admin` / `device` の role、permission、client grant
- Tenant、Connection、Action の設定

Deploy CLI 用の Auth0 Management API 認証情報は CI の secret として注入する。設定ファイルには secret を保存しない。

公式の Auth0 CLI は、Tenant の確認や個別リソースの操作に使用できる。Tenant 構成を宣言的に export/import する用途は Deploy CLI に統一する。

## JWT 検証（推奨）

Auth0 の OIDC discovery と JWKS を利用し、API サーバーで access token をローカル検証する。Auth0 の推奨設定に合わせ、次を検証する。

Rust では `jsonwebtoken` で JWT を検証し、`reqwest` で JWKS を取得する。JWKS はメモリにキャッシュし、未知の `kid` を受け取った場合に再取得する。

- `iss`
- `aud`
- `exp`、`nbf`
- `sub`
- 許可する署名アルゴリズム（RS256）
- JWKS の `kid`

token の検証後、`permissions`、`azp`、token の主体情報を XLAIR 内部の principal に変換する。

## GitHub Connection の設定

Auth0 Deploy CLI の構成には GitHub Connection と `xlair-dev` membership を確認する Post-Login Action を含める。GitHub OAuth App の client ID と client secret は GitHub Actions secrets に `AUTH0_GITHUB_CLIENT_ID` と `AUTH0_GITHUB_CLIENT_SECRET` として登録する。Action は GitHub App の installation token を発行し、Auth0 user の GitHub numeric user ID から login 名を解決して `GET /orgs/xlair-dev/members/{username}` を確認する。レスポンスが `204` の場合だけログインを許可する。

GitHub App の App ID、base64 化した private key、installation ID を `GITHUB_APP_ID`、`GITHUB_APP_PRIVATE_KEY_BASE64`、`GITHUB_APP_INSTALLATION_ID` として GitHub Actions secrets に登録する。private key はリポジトリに保存しない。

## 未確定事項

- XLAIR 側の endpoint・フィールド単位の認可ポリシー
- JWT 検証に使用する Rust ライブラリ
- 筐体 credential を Deploy CLI の構成管理と provisioning のどこまで一体化するか
