# V1Per Servicing Toolkit - Account & Credit System Roadmap

## Overview

Account-based licensing system with HWID binding and credit-based operations.
Users register with username/password, toolkit binds to their CPU ID.
Credits are managed by admin through a web panel.

---

## Architecture

```
Toolkit (Tauri v2)  <--API-->  Backend (Node.js/Go)  <--DB-->  PostgreSQL
     |                              |
  login/register              auth + credits
  check credits               HWID validation
  deduct credits              operation logs
  send logs                   user management
```

---

## HWID Strategy

Single hardware identifier: **CPU ProcessorID**

Source: `Win32_Processor.ProcessorId`
Example: `BFEBFBFF000906EA`

Why CPU ID:
- Burned into silicon at factory
- Not modifiable via BIOS/software
- Requires physical CPU replacement to change
- Available on every Windows system
- Survives OS reinstall and format

Hash for storage: `SHA256(cpuId)` -> first 16 chars

---

## User Flow

### First Launch (No Account)
1. Toolkit checks for cached session token
2. No token found -> show login/register screen
3. User clicks "Create Account"
4. Fills: username + password
5. Toolkit reads CPU ID via WMI
6. Sends POST /api/register with: username, password, hwid
7. API hashes password (bcrypt), stores user record
8. Returns JWT token
9. Toolkit caches token locally
10. Shows: "Account created. Waiting for credits."
11. Main tool loads in read-only mode (free features only)

### Returning User (Has Account)
1. Toolkit reads cached token
2. Sends GET /api/verify with token
3. API validates token + checks HWID matches
4. If HWID mismatch -> "Account bound to another device"
5. If valid -> returns credits count
6. If credits > 0 -> full access
7. If credits = 0 -> warning banner, free features only

### Operation Execution
1. User clicks an operation (unlock, flash, root)
2. Toolkit checks credits >= operation cost
3. If insufficient -> "Not enough credits"
4. If sufficient -> sends POST /api/deduct with hwid, operation, amount
5. API deducts credits, logs operation
6. Toolkit executes operation
7. Toolkit sends result to POST /api/log

---

## Credit Costs

| Operation          | Credits |
|--------------------|---------|
| Unlock Bootloader  | 5       |
| Flash Firmware     | 3       |
| Root Device        | 5       |
| Unisoc Unlock      | 5       |
| Custom ADB Cmd     | 1       |
| Custom Fastboot Cmd| 1       |
| Driver Download    | 0       |
| Device Detection   | 0       |

---

## API Endpoints

### Authentication
```
POST   /api/register     - Create new account
POST   /api/login        - Login with credentials
GET    /api/verify       - Validate session token
POST   /api/logout       - Invalidate token
```

### Credits
```
GET    /api/credits      - Get credit balance for HWID
POST   /api/deduct       - Deduct credits for operation
```

### Logging
```
POST   /api/log          - Submit operation log
GET    /api/logs/:hwid   - Get logs for user
```

### Admin Only
```
GET    /api/admin/users           - List all users
POST   /api/admin/credits         - Add/remove credits to HWID
POST   /api/admin/ban             - Ban/unban HWID
GET    /api/admin/logs            - All operation logs
GET    /api/admin/stats           - Dashboard statistics
```

---

## Database Schema

### users
```
id          SERIAL PRIMARY KEY
username    VARCHAR(50) UNIQUE NOT NULL
password    VARCHAR(255) NOT NULL (bcrypt hash)
hwid        VARCHAR(64) NOT NULL
credits     INTEGER DEFAULT 0
status      VARCHAR(20) DEFAULT 'active'
created_at  TIMESTAMP DEFAULT NOW()
last_login  TIMESTAMP
```

### operation_logs
```
id          SERIAL PRIMARY KEY
user_id     INTEGER REFERENCES users(id)
operation   VARCHAR(50) NOT NULL
device      VARCHAR(100)
result      VARCHAR(20) -- success/failed
credits_used INTEGER
created_at  TIMESTAMP DEFAULT NOW()
```

### credit_transactions
```
id          SERIAL PRIMARY KEY
user_id     INTEGER REFERENCES users(id)
amount      INTEGER NOT NULL
type        VARCHAR(20) -- add/deduct
reason      TEXT
admin_id    VARCHAR(50)
created_at  TIMESTAMP DEFAULT NOW()
```

---

## Admin Panel Pages

### Dashboard
- Total users count
- Active users (logged in last 7 days)
- Total credits used this month
- Today's operations count
- Top 5 most popular devices
- Revenue summary (if selling credits)

### Users
- Searchable list of all users
- Columns: username, hwid, credits, status, last active
- Click user -> view details + manage credits
- Actions: add credits, remove credits, ban, unban

### Credit Management
- Select user from list
- Enter credit amount
- Select action: add or remove
- Enter reason (required for audit)
- Confirm action
- Log saved automatically

### Operation Logs
- Filterable by user, operation type, date range
- Columns: time, user, operation, device, result, credits used
- Export to CSV

### Settings
- Change admin password
- Set default credits for new users
- Configure credit costs per operation
- API key management

---

## Toolkit Integration

### Startup Sequence
```
1. protection::init()           -- anti-debug check
2. resources::extract_all()     -- extract embedded tools
3. Check cached session token
4. If no token -> show LoginScreen
5. If token -> GET /api/verify
6. If valid -> check credits
7. If credits > 0 -> load main tool
8. If credits = 0 -> load with warning
```

### Login Screen
- Username field
- Password field
- [Login] button
- [Create Account] link
- [Forgot Password] link (opens browser to admin panel)
- Status message area

### Create Account Screen
- Username field
- Password field
- Confirm password field
- [Create Account] button
- [Back to Login] link

### Credit Warning Banner
- Shows when credits = 0
- "You have 0 credits. Contact admin to purchase."
- Does not block free features (driver download, device detection)

### Operation Credit Check
- Before each paid operation
- Show: "This operation costs X credits. You have Y credits."
- [Confirm] or [Cancel]
- If insufficient: "Not enough credits" message

---

## Security Measures

### Password
- Bcrypt hashing with salt
- Minimum 8 characters
- No plain text storage

### Tokens
- JWT with 30-day expiry
- Stored in app data directory
- Invalidated on logout

### HWID Validation
- Checked on every API call
- Mismatch = force re-login
- Admin can approve HWID changes (hardware upgrade)

### Rate Limiting
- 5 login attempts per minute per IP
- 100 API calls per minute per user
- 10 operation attempts per hour per user

### Anti-Tampering
- Toolkit binary integrity check (CRC32 of exe)
- Obfuscated API endpoints
- Encrypted local token storage

---

## Implementation Phases

### Phase 1: Backend Foundation
- Set up Node.js/Go API server
- PostgreSQL database setup
- User registration/login endpoints
- JWT authentication middleware
- HWID generation endpoint
- Basic admin panel (users list)

### Phase 2: Credit System
- Credit balance endpoints
- Credit deduction logic
- Credit transaction logging
- Admin credit management UI

### Phase 3: Toolkit Integration
- Login/register screens in Tauri
- JWT token storage
- HWID collection via WMI
- Credit check before operations
- Operation logging to API

### Phase 4: Admin Panel
- Dashboard with stats
- User management (search, ban, credits)
- Operation logs viewer
- Credit cost configuration
- CSV export

### Phase 5: Security Hardening
- Rate limiting
- Anti-tamper checks
- Encrypted local storage
- HWID change approval flow

---

## Tech Stack

### Backend
- Runtime: Node.js or Go
- Database: PostgreSQL
- Auth: JWT (jsonwebtoken)
- Password: bcrypt
- API: REST or GraphQL

### Admin Panel
- Framework: Next.js or Vue
- UI: Tailwind CSS
- Charts: Chart.js or Recharts
- Auth: Admin JWT

### Toolkit (Tauri v2)
- HWID: WMI queries via Windows API
- HTTP: reqwest (blocking)
- Storage: keyring or encrypted file
- Auth: JWT validation

---

## File Structure

```
toolkit/
  src-tauri/
    src/
      auth.rs              -- HWID generation, token management
      api.rs               -- HTTP client for backend API
  src/renderer/
    pages/
      Auth/
        Login.vue          -- Login screen
        Register.vue       -- Registration screen
      Admin/               -- Admin panel (separate app)
        pages/
          Dashboard.vue
          Users.vue
          Logs.vue
          Settings.vue
  backend/
    src/
      routes/
        auth.js            -- register, login, verify
        credits.js         -- balance, deduct
        logs.js            -- operation logging
        admin.js           -- user management
      middleware/
        auth.js            -- JWT validation
        admin.js           -- admin role check
      db/
        schema.sql         -- database schema
        migrations/        -- schema changes
```

---

## Timeline Estimate

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 1-2 weeks | Working auth + basic admin |
| Phase 2 | 1 week | Credit system working |
| Phase 3 | 1-2 weeks | Toolkit login + credit checks |
| Phase 4 | 1-2 weeks | Full admin panel |
| Phase 5 | 1 week | Security hardening |
| **Total** | **5-8 weeks** | **Full system** |

---

## Notes

- Keep the API simple, no over-engineering
- Admin panel can be a separate deployable app
- Toolkit should degrade gracefully if API is unreachable (free features only)
- Consider offline mode with cached token for limited time
- HWID changes should require manual admin approval
- All credit operations must be logged for audit trail
