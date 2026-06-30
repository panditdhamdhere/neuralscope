const WEAK_SECRETS = new Set([
  "change-me-in-production",
  "dev-secret-change-in-production",
  "change-me",
  "secret",
  "changeme",
]);

export async function register() {
  if (process.env.NEXT_RUNTIME !== "nodejs") {
    return;
  }

  if (process.env.APP_ENV !== "production") {
    return;
  }

  const secret = process.env.BETTER_AUTH_SECRET;
  if (
    !secret ||
    secret.length < 32 ||
    WEAK_SECRETS.has(secret.toLowerCase())
  ) {
    throw new Error(
      "BETTER_AUTH_SECRET must be at least 32 characters and not a placeholder when APP_ENV=production",
    );
  }
}
