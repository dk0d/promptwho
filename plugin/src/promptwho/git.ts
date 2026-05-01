
/** 
 * From: https://github.com/kdcokenny/opencode-worktree
 * 
 * Result type for fallible operations 
 * */
interface OkResult<T> {
  readonly ok: true
  readonly value: T
}
interface ErrResult<E> {
  readonly ok: false
  readonly error: E
}
type Result<T, E> = OkResult<T> | ErrResult<E>

const Result = {
  ok: <T>(value: T): OkResult<T> => ({ ok: true, value }),
  err: <E>(error: E): ErrResult<E> => ({ ok: false, error }),
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
