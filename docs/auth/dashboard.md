# 管理 dashboard の認証

管理 dashboard は Auth0 の `XLAIR Dashboard` Regular Web Application を使用する。

GitHub の認可画面には `XLAIR Login` と表示される。

ログインには Auth0 の GitHub Connection を使用する。Auth0 の Post-Login Action が GitHub App の installation token で `xlair-dev` の membership を確認し、所属しているユーザーだけを許可する。

ログイン開始時は Auth0 の `/authorize` に `connection=github` を指定し、Universal Login の Connection 選択画面を表示せず GitHub へ遷移させる。Dashboard 側のログイン実装には次の要件がある。

```text
https://<AUTH0_ISSUER>/authorize?...&connection=github
```

Auth0 側でも `Username-Password-Authentication` は `XLAIR Dashboard` の enabled clients から除外しているため、メールアドレス・パスワードによるログインは許可しない。

## Auth0 Application の設定

dashboard の公開 URL が決まったら、Auth0 の Application に次を設定する。

- Allowed Callback URLs: dashboard の callback URL
- Allowed Logout URLs: dashboard の logout URL
- Allowed Web Origins: dashboard の origin

Auth0 の設定は [`auth0/tenant.yaml`](../../auth0/tenant.yaml) で管理する。URL は公開 URL 決定後に構成へ追加する。

API サーバーは `azp` が `XLAIR Dashboard` の Client ID と一致するユーザートークンだけを `Admin` principal として扱う。Client ID は `AUTH0_DASHBOARD_CLIENT_ID` に設定する。

## API の呼び出し

ログイン後に取得した Auth0 access token を、API 呼び出しの Bearer token として送信する。

```http
Authorization: Bearer <user access token>
```

access token を要求する際は API の audience を指定する。

```text
https://api.xlair.dev
```
