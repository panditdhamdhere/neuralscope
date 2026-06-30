"use client";

import { GitHistoryPanel } from "@/features/git/git-history-panel";
import { useProjectSession } from "@/hooks/use-project-session";

export default function GitPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Git History</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Commits, diffs, and deployment correlation.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <GitHistoryPanel projectId={projectId} token={token} />
      )}
    </div>
  );
}
