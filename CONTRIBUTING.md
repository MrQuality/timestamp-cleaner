# Development and commit workflow

## Contributions and licensing

Original project material is licensed under Apache-2.0; see [LICENSE](LICENSE).
Submit only work you have the right to contribute under that license. Intentionally
submitted contributions are accepted under Apache-2.0 unless explicitly stated otherwise
and agreed by the maintainers. Preserve applicable third-party notices, and identify any
borrowed code, artwork or new dependencies in the pull request. Do not submit family
footage, credentials or personal data; use synthetic regression fixtures.

For setup and checks, follow [README.md](README.md). Use issues for reproducible bugs
and scoped proposals, and pull requests for reviewed changes. Follow the private reporting
guidance in [SECURITY.md](SECURITY.md) for vulnerabilities.

## Collaboration expectations

Keep technical disagreements focused on code and evidence. Be respectful, assume
good faith and do not harass contributors. Maintainers may close abusive discussions
or remove material that exposes personal data. There is no guaranteed support
response time or supported stable release yet; current development targets main.
A dedicated conduct-reporting contact and broader governance policy remain owner
decisions before growing a contributor community.

Keep personal test recordings outside the checkout or in the ignored `.local-media/`
folder. Environment files and private signing bundles are also ignored; these patterns
are convenience safeguards, not substitutes for review and full-history secret scanning.

Keep commits small and coherent, with enough context to understand the work without the original
conversation. Keep tests with their behavior changes and keep unrelated refactoring separate.
Prefer 100–300 changed source lines when practical; correctness and reviewability take precedence
over a line-count target. The initial application bootstrap is already committed; future work
should use smaller increments without rewriting that history.

## Configure the shared template

The template is tracked in `.gitmessage`. Enable it for each clone from PowerShell:

```powershell
git config --local commit.template (Join-Path (git rev-parse --show-toplevel) '.gitmessage')
```

Then `git commit` opens the shared template. Fill all five sections and remove the commented
prompts. This repository is configured locally; other repositories and global Git settings are
unaffected. Git does not distribute local configuration when someone clones the repository.

Commands using `git commit -m` or `--file` bypass the editor template. Their messages must still
follow the same structure. For multiline automated messages, write the complete message to a
temporary file and pass `--file`, preserving actual newlines.

## Message structure

- **Context:** the problem, motivation or constraint that explains the change.
- **Changes:** the concrete result and consequential design decisions.
- **Validation:** checks actually run and their outcomes; relevant checks not run and why.
- **Progress:** the roadmap milestone advanced, what is complete and remaining limitations.
- **Next:** one concrete continuation step, or an explicit statement that this task is complete.

Use short paragraphs or simple bullets. Scale detail to the change. Avoid restating the diff,
transcribing the conversation, or including secrets and private media paths. Commit dates and
Git history already record chronology; the body should capture reasoning and state.

Example for a small geometry fix:

```text
Keep repair selections aligned when the preview is resized

Context:
Selection coordinates must remain tied to source pixels when the display size changes.

Changes:
Convert pointer positions into normalized coordinates before deriving the source rectangle.

Validation:
Geometry tests pass for forward and reverse drags and selections extending outside the canvas.

Progress:
Mask editor: rectangle selection now remains aligned across display sizes. Brush editing remains open.

Next:
Add source-coordinate brush strokes with undo and redo.
```

The example illustrates the format; it is not evidence of additional implemented features.

## Resuming development

Read recent commit bodies alongside ROADMAP.md and VALIDATION.md, inspect the working tree,
then verify assumptions against the current implementation. Commit messages preserve the context
of an increment; the roadmap tracks product scope and the validation record tracks test evidence.

The security, performance and testing requirements in SECURITY.md and docs/architecture.md remain
part of the acceptance criteria for every relevant implementation change.
