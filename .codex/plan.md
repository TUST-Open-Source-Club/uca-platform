# Plan: VolunteerHours (MVP foundation)

## Goal
Implement the core backend + frontend skeleton for the VolunteerHours system based on functionlist.md, with secure auth (Passkey/TOTP), local file storage, and detailed API docs.

## Steps
1) Context & API design
- Confirm data model and core routes.
- Define auth flows (Passkey/TOTP/recovery code/device management).
- Draft OpenAPI schema.

2) Backend foundation
- Set up Axum app, configuration, error handling, logging.
- Add SeaORM, migrations, and entities for users/roles/students/volunteer/contest/review/attachments/devices/recovery codes.
- Implement RBAC middleware + request validation.

3) Auth & security
- Implement Passkey (WebAuthn) + TOTP verification.
- Enforce 2FA at login; add device management + recovery code flows.
- TLS support: default self-signed cert; allow import encrypted key.

4) Feature APIs
- Student import, query, and search.
- Volunteer/contest submissions with attachments.
- Review workflow (self/initial/final, signatures).
- Export PDF/Excel endpoints.

5) Frontend skeleton
- Vue3 routes & layouts for login/2FA, student portal, reviewer, admin.
- API client scaffolding.

6) Tests & docs
- Unit tests for auth/validation/permissions.
- API docs (OpenAPI + README usage).

## Verification
- Backend: cargo test, run server, basic health checks.
- Frontend: pnpm test:unit, manual route check.

## Rollback
- Revert changed files via git.
