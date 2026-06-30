import type { GitCommit, GitDeployment } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchGitCommits(
  projectId: string,
  params: { branch?: string; search?: string; limit?: number } = {},
  token?: string,
): Promise<{ data: GitCommit[]; meta: { total: number } }> {
  const query = new URLSearchParams();
  if (params.branch) query.set("branch", params.branch);
  if (params.search) query.set("search", params.search);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/git/commits?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch commits: ${response.status}`);
  }

  return response.json();
}

export async function fetchGitDeployments(
  projectId: string,
  params: { environment?: string; limit?: number } = {},
  token?: string,
): Promise<{ data: GitDeployment[]; meta: { total: number } }> {
  const query = new URLSearchParams();
  if (params.environment) query.set("environment", params.environment);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/git/deployments?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch deployments: ${response.status}`);
  }

  return response.json();
}

export async function ingestGitCommit(
  projectId: string,
  body: {
    sha: string;
    author: string;
    message: string;
    branch?: string;
  },
  token?: string,
): Promise<GitCommit> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/git/commits`,
    {
      method: "POST",
      headers: authHeaders(token),
      body: JSON.stringify(body),
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to ingest commit: ${response.status}`);
  }

  return response.json();
}

export async function ingestGitDeployment(
  projectId: string,
  body: {
    commitSha: string;
    environment: string;
    deployedBy?: string;
  },
  token?: string,
): Promise<GitDeployment> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/git/deployments`,
    {
      method: "POST",
      headers: authHeaders(token),
      body: JSON.stringify({
        commit_sha: body.commitSha,
        environment: body.environment,
        deployed_by: body.deployedBy,
      }),
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to ingest deployment: ${response.status}`);
  }

  return response.json();
}
