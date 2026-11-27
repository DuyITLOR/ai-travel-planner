// PostToolUse hook: format the file Claude just edited, the way `pnpm format` would.
// Rust goes through rustfmt, everything else through Prettier (which honours .prettierignore).
// A formatter that is missing or fails never blocks the edit: `pnpm format:check` is the gate.

import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { editedFile, repoOf } from './lib.mjs';

const PRETTIER = /\.(ts|tsx|js|jsx|mjs|cjs|json|css|md|ya?ml)$/;

const file = editedFile();
const repo = file && existsSync(file) ? repoOf(file) : null;

if (repo && !repo.path.startsWith('..')) {
  const run = (cmd, args, cwd) =>
    execFileSync(cmd, args, { cwd, stdio: 'ignore', shell: process.platform === 'win32' });
  try {
    if (repo.path.startsWith('apps/api/') && repo.path.endsWith('.rs')) {
      // From apps/api so rust-toolchain.toml picks the pinned rustfmt.
      run('rustfmt', ['--edition', '2024', file], join(repo.root, 'apps/api'));
    } else if (PRETTIER.test(repo.path)) {
      run('pnpm', ['exec', 'prettier', '--write', '--ignore-unknown', file], repo.root);
    }
  } catch {
    // Not installed, or the file does not parse yet: leave it to the check commands.
  }
}
