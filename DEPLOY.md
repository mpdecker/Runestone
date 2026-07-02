# Runestone â€” deployment

_Last updated: 2026-07-01 (Phase 2 readiness pass)_

## Stack

other project under `C:\Development\Runestone`

## Prerequisites

- CI workflow: `.github/workflows/ci.yml` (if present)
- Copy `.env.example` â†’ `.env.local` / host secrets

## Environment

_No .env.example in repo â€” configure secrets in host dashboard._

## Local dev

```bash
cd C:\Development\Runestone
npm ci
# no dev script â€” see README
```

## Build & test

```bash
# no test script
# no build script
```

## Host

Vercel (Next/Vite web) or static hosting per README

## Smoke check

- [ ] Local dev server starts without env errors
- [ ] Test command exits 0 (or documented skip reason in READINESS.md)
- [ ] Production URL / store build succeeds

## Rollback

Redeploy the previous host build (Vercel promotion rollback, EAS prior build, or Docker image tag).
