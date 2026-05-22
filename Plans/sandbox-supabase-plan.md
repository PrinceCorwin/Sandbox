# Sandbox Backend Plan — Supabase for SQL-based Mini-apps

## Decision
Use **Supabase free tier** as the backend for any Sandbox mini-app or personal app (Vantage-style or otherwise) that needs SQL-style tabular data with cross-device sync.

## Why this came up
Originally explored hosting SQL Server on AmalfiNAS. Eliminated because:
- Sandbox runs on the work laptop, and accessing NAS from work requires Tailscale (not desired due to Comfort Systems USA umbrella tooling visibility) or Cloudflare Tunnel (adds external attack surface, requires running cloudflared on work machine).
- Personal Azure was rejected for cost-creep risk and another paid account to manage.
- Synology Drive folder-sync of SQLite files is unsafe (sync clients don't respect DB file locks → corruption).

## Why Supabase
- **Hard-capped free tier** — project suspends at limits, no surprise bills.
  - 500 MB Postgres DB
  - 50k monthly active users
  - 5 GB bandwidth/month
  - 1 GB file storage
  - Daily backups included
  - No credit card required to start
- **Postgres backend** — real SQL, joins, aggregations, full-text search, JSON columns.
- **Auto-generated REST and GraphQL APIs** — define a table, the API exists. No backend code needed for CRUD.
- **Auth built in** — email/password, OAuth, magic links. Row-level security ties user identity to row access at the DB layer (same security model as Firestore rules, written in SQL).
- **Realtime via WebSockets** — subscribe to a table or query, get push updates on changes.
- **HTTPS-only traffic** — works from any network without VPN. Looks like normal web traffic to corporate monitoring.

## Standard architecture for a Sandbox mini-app
```
┌─────────────────────────────┐    HTTPS    ┌──────────────────────┐
│  Sandbox client (WPF/Tauri/ │ ──────────► │  Supabase            │
│  mobile/whatever)           │             │  - Postgres DB       │
│  - supabase-csharp SDK      │ ◄────────── │  - Auth              │
│  - Or raw HttpClient        │  WebSocket  │  - REST/GraphQL API  │
│                             │             │  - Realtime channel  │
└─────────────────────────────┘             └──────────────────────┘
```

Single Supabase project can host many mini-apps. Use Postgres schemas or table prefixes (`ilist_`, `vantage_`, `tools_`) to keep them organized.

## .NET integration
- Primary SDK: `supabase-csharp` (NuGet: `supabase`). Wraps auth, queries, realtime.
- Fallback: raw `HttpClient` against the auto-generated REST endpoints. Works fine for simple CRUD.
- Direct Postgres connection: also possible via `Npgsql` if a use case ever needs raw SQL access. Connection details are in the Supabase project settings.

## What's NOT in scope
- **iList stays on Firebase.** It's working. NoSQL fits its hierarchical list/item shape. No upside to migrating.
- **VANTAGE production data stays on company Azure.** This plan is for personal/dev work only. Any company data goes through company-approved infrastructure.
- **Self-hosting Supabase on AmalfiNAS.** Possible (it's open source) but defeats the entire reason for picking a cloud BaaS in the first place.

## Free tier discipline (avoid surprise suspensions)
- Keep total DB size across all mini-apps under ~400 MB to leave headroom.
- Don't store large binary data (images, files) directly in Postgres — use Supabase Storage with its own free tier or just keep them out of the system.
- Monitor monthly bandwidth in the dashboard if any app gets heavy use.

## Next steps when picking this up
1. Create a Supabase account at supabase.com (free, no card).
2. Create one project — name it `sandbox` or similar. Pick a region close to Houston (us-east-2 / Ohio is the closest US East option; us-west-1 / Oregon is the closest US West).
3. Save the project URL and the `anon` public API key for client use.
4. Save the `service_role` key somewhere secure — bypasses RLS, never put it in client code.
5. For the first mini-app: define the table schema in the Supabase SQL editor, enable Row-Level Security on each table, write a basic RLS policy (typically `auth.uid() = user_id` to scope rows to the logged-in user).
6. Wire the supabase-csharp SDK into the Sandbox app and test against a throwaway table.

## Context for future Claude
- User has clinical memory issues — restate context, don't assume recall.
- User prefers direct communication, senior-dev level, no padding.
- One step at a time during implementation.
- Push back on assumptions when warranted, don't just comply.
- This decision came from a long conversation that worked through NAS-hosted, Azure-hosted, Cloudflare Tunnel, and Synology Drive sync options before landing on Supabase. Don't re-litigate those rejections without new information.
