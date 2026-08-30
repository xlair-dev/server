# 管理 dashboard の認証

管理 dashboard は Auth0 の `XLAIR Dashboard` Regular Web Application を使用する。

## Auth0 Application の設定

dashboard の公開 URL が決まったら、Auth0 の Application に次を設定する。

- Allowed Callback URLs: dashboard の callback URL
- Allowed Logout URLs: dashboard の logout URL
- Allowed Web Origins: dashboard の origin

Application は `auth0/tenant.yaml` で管理する。URL は公開 URL 決定後に構成へ追加する。

## API の呼び出し

ログイン後に取得した Auth0 access token を、API 呼び出しの Bearer token として送信する。

```http
Authorization: Bearer <user access token>
```

access token を要求する際は API の audience を指定する。

```text
https://api.xlair.dev
```
