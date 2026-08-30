# Auth0 configuration

This directory contains the Auth0 Tenant configuration managed by Auth0 Deploy CLI (`a0deploy`).

The repository uses one Auth0 Tenant:

```text
dev-2dn3mvmvr8tccoss.us.auth0.com
```

## Local usage

Install the pinned Deploy CLI version and provide the following environment variables:

```text
AUTH0_DOMAIN=dev-2dn3mvmvr8tccoss.us.auth0.com
AUTH0_CLIENT_ID=<deploy-cli-client-id>
AUTH0_CLIENT_SECRET=<deploy-cli-client-secret>
```

Preview the changes:

```sh
a0deploy import \
  --config_file=auth0/config.json \
  --input_file=auth0/tenant.yaml \
  --dry-run
```

Apply the changes after reviewing the preview:

```sh
a0deploy import \
  --config_file=auth0/config.json \
  --input_file=auth0/tenant.yaml \
  --dry-run \
  --apply
```

The Deploy CLI Management API client should be dedicated to configuration deployment and granted only the Management API permissions required by the resources managed here. Its credentials are supplied through the environment or CI secrets.

The deployment workflow runs automatically after changes under `auth0/` reach `main`. Deletions remain disabled by `AUTH0_ALLOW_DELETE=false`.

## Configuration model

The XLAIR API defines two coarse permissions:

- `admin`: administrator principal
- `device`: device principal

The `admin` role contains the `admin` permission. Device access is granted to the shared M2M Application through a client grant. Endpoint and field-level authorization remains an XLAIR implementation concern.

The dashboard Application and the shared device M2M Application will be added after their callback URL and client-grant configuration are finalized.
