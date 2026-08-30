const ORGANIZATION = "xlair-dev";
const AUTH0_DOMAIN = "dev-2dn3mvmvr8tccoss.us.auth0.com";

const deny = (api) => {
  api.access.deny("GitHub organization membership could not be verified.");
};

const getIdentityProviderAccessToken = async (event) => {
  let tokenResponse;
  try {
    tokenResponse = await fetch(`https://${AUTH0_DOMAIN}/oauth/token`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        grant_type: "client_credentials",
        client_id: event.secrets.AUTH0_MANAGEMENT_CLIENT_ID,
        client_secret: event.secrets.AUTH0_MANAGEMENT_CLIENT_SECRET,
        audience: `https://${AUTH0_DOMAIN}/api/v2/`,
      }),
    });
  } catch (_error) {
    console.log("Auth0 Management API token request threw an exception");
    return null;
  }

  if (!tokenResponse.ok) {
    console.log(`Auth0 Management API token request failed: ${tokenResponse.status}`);
    return null;
  }

  let token;
  try {
    token = await tokenResponse.json();
  } catch (_error) {
    console.log("Auth0 Management API token response could not be parsed");
    return null;
  }

  let userResponse;
  try {
    userResponse = await fetch(
      `https://${AUTH0_DOMAIN}/api/v2/users/${encodeURIComponent(event.user.user_id)}?fields=identities&include_fields=true`,
      { headers: { Authorization: `Bearer ${token.access_token}` } },
    );
  } catch (_error) {
    console.log("Auth0 user profile request threw an exception");
    return null;
  }

  if (!userResponse.ok) {
    console.log(`Auth0 user profile request failed: ${userResponse.status}`);
    return null;
  }

  let user;
  try {
    user = await userResponse.json();
  } catch (_error) {
    console.log("Auth0 user profile response could not be parsed");
    return null;
  }
  return user.identities?.find(({ provider }) => provider === "github")?.access_token;
};

exports.onExecutePostLogin = async (event, api) => {
  let accessToken;
  try {
    accessToken = await getIdentityProviderAccessToken(event);
  } catch (_error) {
    console.log("GitHub organization membership request threw an exception");
    deny(api);
    return;
  }

  if (!accessToken) {
    deny(api);
    return;
  }

  let response;
  try {
    response = await fetch(
      `https://api.github.com/user/memberships/orgs/${ORGANIZATION}`,
      {
        headers: {
          Accept: "application/vnd.github+json",
          Authorization: `Bearer ${accessToken}`,
          "X-GitHub-Api-Version": "2022-11-28",
        },
      },
    );
  } catch (_error) {
    console.log("GitHub organization membership response could not be parsed");
    deny(api);
    return;
  }

  if (!response.ok) {
    console.log(`GitHub organization membership request failed: ${response.status}`);
    deny(api);
    return;
  }

  let membership;
  try {
    membership = await response.json();
  } catch (_error) {
    deny(api);
    return;
  }

  if (membership.state !== "active") {
    api.access.deny("An active XLAIR GitHub organization membership is required.");
  }
};
