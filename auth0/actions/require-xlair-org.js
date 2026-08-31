const ORGANIZATION = "xlair-dev";
const AUTH0_DOMAIN = "dev-2dn3mvmvr8tccoss.us.auth0.com";

const deny = (api) => {
  api.access.deny("GitHub organization membership could not be verified.");
};

const describeError = (error) => {
  const value = (item) => String(item ?? "unknown").slice(0, 80);
  return [
    `name=${value(error?.name)}`,
    `message=${value(error?.message)}`,
    `cause_code=${value(error?.cause?.code)}`,
    `cause=${value(error?.cause?.message)}`,
  ].join(" ");
};

const getIdentityProviderAccessToken = async (event) => {
  console.log(
    `stage=start management_client_id=${event.secrets.AUTH0_MANAGEMENT_CLIENT_ID}`,
  );

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
  } catch (error) {
    console.log(`stage=management-token exception ${describeError(error)}`);
    return null;
  }

  console.log(`stage=management-token status=${tokenResponse.status}`);

  if (!tokenResponse.ok) {
    return null;
  }

  let token;
  try {
    token = await tokenResponse.json();
  } catch (_error) {
    console.log("stage=management-token parse_error");
    return null;
  }

  let userResponse;
  try {
    userResponse = await fetch(
      `https://${AUTH0_DOMAIN}/api/v2/users/${encodeURIComponent(event.user.user_id)}?fields=identities&include_fields=true`,
      { headers: { Authorization: `Bearer ${token.access_token}` } },
    );
  } catch (error) {
    console.log(`stage=user-profile exception ${describeError(error)}`);
    return null;
  }

  console.log(`stage=user-profile status=${userResponse.status}`);

  if (!userResponse.ok) {
    return null;
  }

  let user;
  try {
    user = await userResponse.json();
  } catch (_error) {
    console.log("stage=user-profile parse_error");
    return null;
  }

  const githubIdentity = user.identities?.find(
    ({ provider }) => provider === "github",
  );
  console.log(
    `stage=idp-token github_identity=${Boolean(githubIdentity)} access_token=${Boolean(githubIdentity?.access_token)}`,
  );
  return githubIdentity?.access_token;
};

exports.onExecutePostLogin = async (event, api) => {
  let accessToken;
  try {
    accessToken = await getIdentityProviderAccessToken(event);
  } catch (error) {
    console.log(`stage=github-membership exception ${describeError(error)}`);
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
    console.log(`stage=github-membership status=${response.status}`);
    deny(api);
    return;
  }

  let membership;
  try {
    membership = await response.json();
  } catch (_error) {
    console.log("stage=github-membership parse_error");
    deny(api);
    return;
  }

  console.log(`stage=github-membership status=${response.status}`);
  console.log(`stage=github-membership state=${membership.state}`);

  if (membership.state !== "active") {
    api.access.deny("An active XLAIR GitHub organization membership is required.");
  }
};
