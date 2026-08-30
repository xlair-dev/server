const ORGANIZATION = "xlair-dev";

exports.onExecutePostLogin = async (event, api) => {
  const identity = event.user.identities?.find(
    ({ provider }) => provider === "github",
  );
  const accessToken = identity?.access_token;

  if (!accessToken) {
    api.access.deny("GitHub organization membership could not be verified.");
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
    api.access.deny("GitHub organization membership could not be verified.");
    return;
  }

  if (!response.ok) {
    api.access.deny("GitHub organization membership could not be verified.");
    return;
  }

  let membership;
  try {
    membership = await response.json();
  } catch (_error) {
    api.access.deny("GitHub organization membership could not be verified.");
    return;
  }

  if (membership.state !== "active") {
    api.access.deny("An active XLAIR GitHub organization membership is required.");
  }
};
