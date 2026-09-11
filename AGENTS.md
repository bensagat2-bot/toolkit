# AGENTS.md
Guidance for OpenCode (and other AI agents) working with code in this repo.

## Project Context
Free Android servicing toolkit. Focus on practical device repair, diagnostics, firmware, ADB/fastboot utilities, and technician workflows. Keep it lightweight, reliable, and free.

## General Principles
- Generate concise, short solutions for new modules or code.
- Watch for over-engineering and oversized files that need refactoring.
- Watch for weird syntax/style that mismatches the rest of the codebase.
- Watch for obvious bugs and the blast radius of errors.
- No emojis or special characters in comments.
- Write activity-log.md in /docs to refer back if confused.
- Run major changes by the user first — do not execute blindly.
- Review existing files before any refactor or change.
- Markdown files use kebab-case naming (example: some-description-changes.md).
- Comments: one-liner, one sentence.

## Code Quality
- Choose the right data structures and algorithms for the problem.
- Don't expose data needlessly (least privilege).
- No external libraries unless absolutely necessary.
- Use the project dependency file for correct versions.
- Avoid redundancy unless it clearly improves usability.
- Prefer pure functions and clear separation between UI, core logic, and device communication layers.

## Android / Toolkit Specific
- Prefer ADB, fastboot, and standard Android tools over proprietary closed binaries when possible.
- Keep device communication code isolated and well-logged.
- Handle connection drops, permission denials, and device state changes gracefully.
- Never hardcode credentials, tokens, or activation keys.
- Document supported device families and known limitations clearly.
- Prefer offline-first design; network should be optional.

## Version Control
- Commit after significant changes with clear messages.
- Keep commits focused and atomic.
- No auto-push to any branch.
- Don't auto-commit activity logs and docs.
- Access only these repositories: <REPO_ALLOWLIST>

## AI Restrictions
- No customer personal data — names, contacts, account numbers, transactions (unless approved exemption).
- No credentials — passwords, API keys, tokens, connection strings.
- Always check that any npm/yarn/pip/cargo install or download is safe. Verify via <PACKAGE_REGISTRY_HOST>.
- Do not generate code that enables unauthorized access, theft of accounts, or bypass of security for devices the user does not own or have explicit permission to service.

## Safety & Ethics
- This toolkit is intended for legitimate device repair, diagnostics, and recovery on devices the technician owns or has authorization to service.
- Refuse requests that clearly target unauthorized access, fraud, or circumvention of security for devices without permission.