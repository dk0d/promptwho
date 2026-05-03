import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import { Result } from "./result";

export interface GitSnapshot {
  branch?: string;
  head_commit?: string;
  dirty: boolean;
  staged_files: string[];
  unstaged_files: string[];
}

export interface GitProjectIdentity {
  repositoryFingerprint: string;
}


/**
 * From: https://github.com/kdcokenny/opencode-worktree
 * 
 * Execute a git command safely using Bun.spawn with explicit array.
 * Avoids shell interpolation entirely by passing args as array.
 */
async function git(args: string[], cwd: string): Promise<Result<string, string>> {
  try {
    const proc = Bun.spawn(["git", ...args], {
      cwd,
      stdout: "pipe",
      stderr: "pipe",
    })
    const [stdout, stderr, exitCode] = await Promise.all([
      new Response(proc.stdout).text(),
      new Response(proc.stderr).text(),
      proc.exited,
    ])
    if (exitCode !== 0) {
      return Result.err(stderr.trim() || `git ${args[0]} failed`)
    }
    return Result.ok(stdout.trim())
  } catch (error) {
    return Result.err(error instanceof Error ? error.message : String(error))
  }
}

function parseStatusEntries(status: string): {
  dirty: boolean;
  staged_files: string[];
  unstaged_files: string[];
} {
  const staged = new Set<string>();
  const unstaged = new Set<string>();

  for (const line of status.split("\n")) {
    if (!line.trim() || line.length < 4) {
      continue;
    }

    const indexStatus = line[0];
    const worktreeStatus = line[1];
    const file = line.slice(3).trim();

    if (!file) {
      continue;
    }

    if (indexStatus !== " " && indexStatus !== "?") {
      staged.add(file);
    }

    if (worktreeStatus !== " ") {
      unstaged.add(file);
    }

    if (indexStatus === "?" && worktreeStatus === "?") {
      unstaged.add(file);
    }
  }

  return {
    dirty: staged.size > 0 || unstaged.size > 0,
    staged_files: [...staged],
    unstaged_files: [...unstaged],
  };
}

export async function getGitSnapshot(cwd: string): Promise<GitSnapshot | null> {
  const insideWorkTree = await git(["rev-parse", "--is-inside-work-tree"], cwd);
  if (!insideWorkTree.ok || insideWorkTree.value !== "true") {
    return null;
  }

  const [branch, headCommit, status] = await Promise.all([
    git(["branch", "--show-current"], cwd),
    git(["rev-parse", "HEAD"], cwd),
    git(["status", "--porcelain"], cwd),
  ]);

  const parsedStatus = status.ok
    ? parseStatusEntries(status.value)
    : {
      dirty: false,
      staged_files: [],
      unstaged_files: [],
    };

  return {
    branch: branch.ok && branch.value ? branch.value : undefined,
    head_commit: headCommit.ok && headCommit.value ? headCommit.value : undefined,
    dirty: parsedStatus.dirty,
    staged_files: parsedStatus.staged_files,
    unstaged_files: parsedStatus.unstaged_files,
  };
}

function normalizeRemoteUrl(url: string): string {
  return url
    .trim()
    .replace(/\.git$/i, "")
    .replace(/^git@([^:]+):/, "ssh://git@$1/")
    .replace(/^https?:\/\//i, "https://")
    .toLowerCase();
}

async function fingerprint(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  const hex = [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
  return `git:${hex}`;
}

export async function getGitProjectIdentity(cwd: string): Promise<GitProjectIdentity | null> {
  const insideWorkTree = await git(["rev-parse", "--is-inside-work-tree"], cwd);
  if (!insideWorkTree.ok || insideWorkTree.value !== "true") {
    return null;
  }

  const [remoteUrl, topLevel] = await Promise.all([
    git(["remote", "get-url", "origin"], cwd),
    git(["rev-parse", "--show-toplevel"], cwd),
  ]);

  if (remoteUrl.ok && remoteUrl.value) {
    return {
      repositoryFingerprint: await fingerprint(`remote:${normalizeRemoteUrl(remoteUrl.value)}`),
    };
  }

  if (topLevel.ok && topLevel.value) {
    const repositoryId = await readOrCreateRepositoryId(topLevel.value);
    if (!repositoryId) {
      return null;
    }
    return {
      repositoryFingerprint: await fingerprint(`local:${repositoryId}`),
    };
  }

  return null;
}

async function readOrCreateRepositoryId(repoRoot: string): Promise<string | null> {
  const promptwhoDir = join(repoRoot, ".promptwho");
  const idPath = join(promptwhoDir, "repository-id");

  try {
    const existing = (await readFile(idPath, "utf8")).trim();
    if (existing) {
      return existing;
    }
  } catch (error) {
    if (!(error instanceof Error) || !("code" in error) || error.code !== "ENOENT") {
      return null;
    }
  }

  const repositoryId = crypto.randomUUID();

  try {
    await mkdir(promptwhoDir, { recursive: true });
    await writeFile(idPath, `${repositoryId}\n`, { flag: "wx" });
    return repositoryId;
  } catch (error) {
    if (!(error instanceof Error) || !("code" in error) || error.code !== "EEXIST") {
      return null;
    }
  }

  try {
    const existing = (await readFile(idPath, "utf8")).trim();
    return existing || null;
  } catch {
    return null;
  }
}
