"use client";

import { useCallback, useEffect, useState } from "react";
import { GitBranch, Rocket } from "lucide-react";

import type { GitCommit, GitDeployment } from "@neuralscope/shared";

import { fetchGitCommits, fetchGitDeployments } from "@/services/git";

interface GitHistoryPanelProps {
  projectId?: string;
  token?: string;
}

export function GitHistoryPanel({ projectId, token }: GitHistoryPanelProps) {
  const [commits, setCommits] = useState<GitCommit[]>([]);
  const [deployments, setDeployments] = useState<GitDeployment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();

  const load = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(undefined);

    try {
      const [commitsRes, deploymentsRes] = await Promise.all([
        fetchGitCommits(projectId, { limit: 30 }, token),
        fetchGitDeployments(projectId, { limit: 20 }, token),
      ]);
      setCommits(commitsRes.data);
      setDeployments(deploymentsRes.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load git history");
    } finally {
      setLoading(false);
    }
  }, [projectId, token]);

  useEffect(() => {
    void load();
  }, [load]);

  if (!projectId) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Sign in and create a project to view Git history and deployments.
      </div>
    );
  }

  if (loading) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Loading git history...
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <section className="glass rounded-xl p-6">
        <div className="mb-4 flex items-center gap-2">
          <GitBranch className="h-4 w-4 text-indigo-400" />
          <h2 className="text-sm font-medium text-white">Recent commits</h2>
        </div>

        {commits.length === 0 ? (
          <p className="text-sm text-zinc-500">
            No commits yet. Ingest via{" "}
            <code className="text-zinc-400">POST /api/v1/projects/:id/git/commits</code> or CI webhook.
          </p>
        ) : (
          <div className="divide-y divide-zinc-800/60">
            {commits.map((commit) => (
              <div key={commit.id} className="py-3 first:pt-0 last:pb-0">
                <div className="flex flex-wrap items-center gap-2">
                  <code className="rounded bg-zinc-900 px-2 py-0.5 text-xs text-indigo-300">
                    {commit.sha.slice(0, 7)}
                  </code>
                  <span className="text-xs text-zinc-500">{commit.branch}</span>
                  <span className="text-xs text-zinc-600">
                    {new Date(commit.committedAt).toLocaleString()}
                  </span>
                </div>
                <p className="mt-1 text-sm text-zinc-200">{commit.message}</p>
                <p className="mt-0.5 text-xs text-zinc-500">{commit.author}</p>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="glass rounded-xl p-6">
        <div className="mb-4 flex items-center gap-2">
          <Rocket className="h-4 w-4 text-emerald-400" />
          <h2 className="text-sm font-medium text-white">Deployments</h2>
        </div>

        {deployments.length === 0 ? (
          <p className="text-sm text-zinc-500">No deployments recorded yet.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="text-xs text-zinc-500">
                  <th className="pb-2 pr-4 font-medium">Environment</th>
                  <th className="pb-2 pr-4 font-medium">Commit</th>
                  <th className="pb-2 pr-4 font-medium">Message</th>
                  <th className="pb-2 font-medium">Deployed</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60">
                {deployments.map((deployment) => (
                  <tr key={deployment.id}>
                    <td className="py-2 pr-4 text-zinc-300">{deployment.environment}</td>
                    <td className="py-2 pr-4">
                      <code className="text-xs text-indigo-300">
                        {deployment.commitSha.slice(0, 7)}
                      </code>
                    </td>
                    <td className="py-2 pr-4 text-zinc-400">
                      {deployment.commitMessage ?? "—"}
                    </td>
                    <td className="py-2 text-zinc-500">
                      {new Date(deployment.deployedAt).toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}
