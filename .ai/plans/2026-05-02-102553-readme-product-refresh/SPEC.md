---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — readme-product-refresh

## Problem

The README has become both the product front door and the full operator
reference. That makes first-time evaluation harder because install, native hook
details, notification config, release notes, and developer workflows all compete
for attention in one file.

## Requirements

### MUST

- Keep the README focused on what Foreman is, why it exists, and how to try it.
- Move long-running operator details into durable docs.
- Keep setup, doctor, native hooks, notifications, and keybindings discoverable.
- Preserve current command names and version references.
- Add a VHS tape source for a future README demo artifact.

### SHOULD

- Avoid committing a generated GIF unless it is rendered and validated locally.
- Make the docs index route readers to the new operator reference.
- Keep development and release workflow details behind docs links.

## Constraints

- This is a docs-only slice.
- Do not change runtime behavior.
- Do not invent screenshots or demo output.
