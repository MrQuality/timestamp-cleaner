# Engineering and commit conventions

These instructions apply throughout this repository and supplement the parent's standing instructions.

## Small commits

- Make each commit one coherent, reviewable change. Prefer roughly 100–300 changed source lines;
  this is guidance, not a quota. Generated lockfile changes are reviewed separately from source size.
- Split independent fixes, refactoring, features and dependency upgrades. Keep a behavior change
  and its regression tests together. Do not split a change in a way that leaves a commit broken.
- Commit completed, verified increments during development instead of accumulating the entire
  feature in one final commit. Do not rewrite existing history unless explicitly requested.

## Commit context and progress memory

- Follow `.gitmessage` for every new commit, including automated commits made with `-m` or `--file`.
- Use an imperative subject describing the concrete outcome. Populate Context, Changes,
  Validation, Progress and Next with concise, factual text. No empty sections or template prompts.
- Explain why the change exists and any decision needed to understand it later. Record actual
  verification, including important failures, limitations and checks not run. Never infer CI success
  from the existence of a workflow or describe a planned feature as implemented.
- Tie Progress to the relevant milestone in ROADMAP.md; update the roadmap when its status changes.
  Include remaining constraints and one concrete next step so a later session can resume correctly.
- At the start of resumed work, inspect `git status`, recent commit bodies (`git log -5 --format=full`),
  ROADMAP.md and relevant validation evidence. Treat code and fresh checks as authoritative when
  they disagree with historical notes.
- Review staged changes and run checks appropriate to the change before committing. Documentation
  and Git configuration changes need focused inspection; do not rerun media tests without a reason.

See CONTRIBUTING.md for setup and an example. The template is configured per clone; it does not
automatically populate commit bodies or enforce these conventions.
