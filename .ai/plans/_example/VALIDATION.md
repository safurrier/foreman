---
id: example-validation
title: "Example: Add User Auth — Validation"
description: >
  Example validation log showing how changes were verified.
---

# Validation

## 2026-01-16 — Unit + contract tests

```
mise run check — 45 passed (12 new)
```

New tests:
- `test_jwt_middleware.py` — token validation, expiry, malformed tokens
- `test_auth_decorator.py` — protected/unprotected route behavior
- `test_user_model.py` — create, authenticate, password hashing

## 2026-01-16 — Integration tests

```
mise run verify — 52 passed (7 new integration tests)
```

Tested against ephemeral Postgres via docker-compose.
Artifacts: `test-results/junit.xml` (CI upload, not persisted here)

## 2026-01-17 — Manual E2E

Ran the full login → access protected route → refresh token flow locally:

```bash
curl -X POST localhost:8000/register -d '{"email":"test@test.com","password":"secret"}'
curl -X POST localhost:8000/login -d '{"email":"test@test.com","password":"secret"}'
# → {"token": "eyJ..."}
curl -H "Authorization: Bearer eyJ..." localhost:8000/protected
# → 200 OK
```
