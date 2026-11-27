import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve } from 'node:path';

/** The file a tool call edits, from the hook input Claude Code sends on stdin. */
export function editedFile() {
  let input;
  try {
    input = JSON.parse(readFileSync(0, 'utf8'));
  } catch {
    return null;
  }
  const file = input?.tool_input?.file_path ?? input?.tool_input?.notebook_path;
  if (typeof file !== 'string' || file.length === 0) return null;
  return isAbsolute(file) ? file : resolve(input.cwd ?? process.cwd(), file);
}

/** Nearest directory that exists, so git can run for a file not created yet. */
function existingDir(file) {
  let dir = dirname(file);
  while (!existsSync(dir) && dirname(dir) !== dir) dir = dirname(dir);
  return dir;
}

/**
 * The repository the file lives in, asked from the file's own directory:
 * a git worktree on another branch answers for itself, not for the main checkout.
 */
export function repoOf(file) {
  const git = (...args) =>
    execFileSync('git', ['-C', existingDir(file), ...args], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  try {
    const root = git('rev-parse', '--show-toplevel');
    return {
      root,
      branch: git('branch', '--show-current'),
      path: relative(root, file).split('\\').join('/'),
    };
  } catch {
    return null;
  }
}
