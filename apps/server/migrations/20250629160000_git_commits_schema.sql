-- Git commit history for deployment correlation (M13)

CREATE TABLE git_commits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    sha VARCHAR(40) NOT NULL,
    author VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    branch VARCHAR(255) NOT NULL DEFAULT 'main',
    committed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_id, sha)
);

CREATE INDEX idx_git_commits_project_committed ON git_commits (project_id, committed_at DESC);
CREATE INDEX idx_git_commits_project_branch ON git_commits (project_id, branch);
