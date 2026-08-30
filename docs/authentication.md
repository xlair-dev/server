# 認証・認可設計

## 決定事項

| 主体 | 認証 | XLAIR 上の識別 |
| --- | --- | --- |
| 一般ユーザー | 筐体で読み取った card ID | `users.card` |
| 筐体 | Auth0 M2M access token | Auth0 の client identity |
| 管理者・運用者 | Auth0 user access token | Auth0 の user identity |

- 一般ユーザーは Auth0 にアカウントを作成しない
- card ID を XLAIR 上のユーザー識別子として扱う
- カードの複製による同一ユーザーとしての利用を許容する
- 筐体認証には Auth0 M2M を使用する
- 全筐体で 1 つの M2M Application と credentials を共有する
- 筐体 credentials に定期的な有効期限は設けない
- 管理者・運用者の認証には Auth0 を使用する
- OpenAPI では `userAuth` と `deviceAuth` を使用する
- `/users/{userId}` は筐体と管理者の双方から更新できる
- 更新メソッドは `PATCH` とする

## Auth0 と XLAIR の責務

Auth0 は認証と認証主体の大分類を担当する。XLAIR の endpoint、リソース、フィールド単位の認可は XLAIR 側で管理する。

```text
Auth0
  └─ 認証、token 発行、管理者・筐体の識別

XLAIR
  └─ endpoint、リソース、操作内容に対する認可
```

`admin` と `device` は細かな API permission ではなく、XLAIR が認可ポリシーを選択するための principal の種類として扱う。

Auth0 の role と permission を細かな権限体系として利用すると、`admin` や `device` が持つ XLAIR の操作内容まで Auth0 の設定に依存し、外部の認証基盤にアプリケーションの認可モデルを露出する。role と permission を利用する場合も、大分類とほぼ 1 対 1 の対応に留める。

採用する Auth0 上の表現は次のとおりとする。

```text
admin role
  └─ admin permission

device M2M client grant
  └─ device permission
```

Auth0 には endpoint やフィールド単位の permission を登録しない。`admin` / `device` は操作権限の一覧ではなく、XLAIR が認可ポリシーを選択するための大分類である。role と permission を 1:1 に近づけることで、XLAIR の認可モデルを Auth0 の設定へ依存させない。

API 層は token の `permissions` と token の主体種別を検証し、XLAIR 内部の principal に変換する。`scope` は OAuth の token 上の表現として扱い、アプリケーションの認可モデルには持ち込まない。custom claim は使用しない。

```text
DevicePrincipal { client_id }
UserPrincipal { subject }
```

`device` permission を持つ client credentials token は `DevicePrincipal` に、`admin` permission を持つ user token は `UserPrincipal` の管理者主体に変換する。以降の endpoint・リソース・フィールド単位の認可は XLAIR 側で行う。

## 認証フロー

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

Auth0 の user access token を `userAuth` として送信する。現時点では管理者・運用者用だが、将来の認証対象拡張を考慮して scheme 名は `userAuth` とする。

各 endpoint の認証方式は [openapi.yaml](./openapi.yaml) に定義する。筐体と管理者の双方が利用する操作は、認証方式を OR で指定する。

## 筐体 credentials の運用（推奨）

筐体の登録時に provisioning 処理を実行し、共有する Auth0 M2M Application の credential を設置担当者または筐体へ安全に渡す。XLAIR の公開 API に credential 発行機能は設けない。

client credential の定期的な失効は行わない。access token は短寿命に設定し、紛失・侵害時は Auth0 で対象 client を無効化するか、credential を rotation する。Auth0 で client を無効化しても発行済み token は有効期限まで有効なため、短寿命 token とする。

credential の発行は、共有 M2M Application を初期構成する provisioning とする。公開 API からの自己登録は行わず、Auth0 Deploy CLI または Auth0 Management API を利用する管理用 provisioning 処理で M2M Application と API grant を作成する。

Auth0 が利用するプランで Private Key JWT が利用できる場合は、筐体で生成した鍵ペアの公開鍵を Auth0 に登録し、秘密鍵を筐体外へ出さない方式を推奨する。利用できない場合は Client Secret を secure な provisioning 経路で筐体へ配布する。

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

- `iss`
- `aud`
- `exp`、`nbf`
- `sub`
- 許可する署名アルゴリズム（RS256）
- JWKS の `kid`

JWKS はキャッシュし、未知の `kid` を受け取った場合に鍵を再取得する。token の検証後、`permissions` と token の主体情報を XLAIR 内部の principal に変換する。

## 未確定事項

- XLAIR 側の endpoint・フィールド単位の認可ポリシー
- JWT 検証に使用する Rust ライブラリ
- 筐体 credential を Deploy CLI の構成管理と provisioning のどこまで一体化するか
