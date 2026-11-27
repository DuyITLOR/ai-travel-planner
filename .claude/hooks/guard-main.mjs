// PreToolUse hook: refuse to edit a file of this repo while its checkout is on `main`.
// Everything reaches `main` through a PR (CLAUDE.md, Conventions); `/implement` says the same.
// Exit code 2 blocks the tool call and shows stderr to Claude.

import { editedFile, repoOf } from './lib.mjs';

/** Working files that are not committed, so writing them on `main` is harmless. */
const ALLOWED_ON_MAIN = ['docs/plans/'];

const file = editedFile();
const repo = file && repoOf(file);

if (
  repo &&
  repo.branch === 'main' &&
  !repo.path.startsWith('..') &&
  !ALLOWED_ON_MAIN.some((prefix) => repo.path.startsWith(prefix))
) {
  console.error(
    `Blocked: ${repo.path} is being edited on \`main\`. ` +
      'Create a branch first (git switch -c feat/<issue>-<slug>, fix/... or chore/...) and retry.',
  );
  process.exit(2);
}
