const crypto = require("crypto");

const ORGANIZATION = "xlair-dev";
const AUTH0_DOMAIN = "dev-2dn3mvmvr8tccoss.us.auth0.com";
const GITHUB_API_VERSION = "2026-03-10";
const RETURN_TO = "http://localhost:3000";

const rejectAndLogout = (event, api) => {
  // Clear the Auth0 SSO session so the next login starts a fresh IdP authorization.
  api.redirect.sendUserTo(`https://${AUTH0_DOMAIN}/v2/logout`, {
    query: {
      client_id: event.client.client_id,
      returnTo: RETURN_TO,
    },
  });
};

const base64UrlEncode = (value) =>
  Buffer.from(value)
    .toString("base64")
    .replace(/=/g, "")
    .replace(/\+/g, "-")
    .replace(/\//g, "_");

const createGitHubAppJwt = (event) => {
  const now = Math.floor(Date.now() / 1000);
  const header = base64UrlEncode(JSON.stringify({ alg: "RS256", typ: "JWT" }));
  const payload = base64UrlEncode(
    JSON.stringify({
      iat: now - 60,
      exp: now + 540,
      iss: event.secrets.XLAIR_GITHUB_APP_CLIENT_ID,
    }),
  );
  const unsignedToken = `${header}.${payload}`;
  const signature = crypto
    .createSign("RSA-SHA256")
    .update(unsignedToken)
    .sign(
      Buffer.from(event.secrets.XLAIR_GITHUB_APP_PRIVATE_KEY_BASE64, "base64"),
    );

  return `${unsignedToken}.${base64UrlEncode(signature)}`;
};

const getInstallationAccessToken = async (event) => {
  const response = await fetch(
    `https://api.github.com/app/installations/${event.secrets.XLAIR_GITHUB_APP_INSTALLATION_ID}/access_tokens`,
    {
      method: "POST",
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${createGitHubAppJwt(event)}`,
        "X-GitHub-Api-Version": GITHUB_API_VERSION,
      },
    },
  );

  if (!response.ok) {
    return null;
  }

  const body = await response.json();
  return body.token;
};

const getGitHubUserId = (event) => {
  const [provider, userId] = event.user.user_id.split("|", 2);
  return provider === "github" && /^\d+$/.test(userId) ? userId : null;
};

const getGitHubLogin = async (userId, accessToken) => {
  const response = await fetch(`https://api.github.com/user/${userId}`, {
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${accessToken}`,
      "X-GitHub-Api-Version": GITHUB_API_VERSION,
    },
  });

  if (!response.ok) {
    return null;
  }

  const body = await response.json();
  return typeof body.login === "string" ? body.login : null;
};

const isOrganizationMember = async (login, accessToken) => {
  const response = await fetch(
    `https://api.github.com/orgs/${ORGANIZATION}/members/${encodeURIComponent(login)}`,
    {
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${accessToken}`,
        "X-GitHub-Api-Version": GITHUB_API_VERSION,
      },
    },
  );

  return response.status === 204;
};

exports.onExecutePostLogin = async (event, api) => {
  try {
    const userId = getGitHubUserId(event);
    if (!userId) {
      rejectAndLogout(event, api);
      return;
    }

    const accessToken = await getInstallationAccessToken(event);
    if (!accessToken) {
      rejectAndLogout(event, api);
      return;
    }

    const login = await getGitHubLogin(userId, accessToken);
    if (!login || !(await isOrganizationMember(login, accessToken))) {
      rejectAndLogout(event, api);
    }
  } catch (_error) {
    rejectAndLogout(event, api);
  }
};
