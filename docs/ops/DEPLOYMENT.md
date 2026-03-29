# Deployment Procedures

## Pre-Deployment Checklist

- [ ] Code reviewed and approved
- [ ] Tests passing (`cargo test`)
- [ ] Database migrations tested
- [ ] Environment variables configured
- [ ] Backup created
- [ ] Rollback plan prepared

---

## Deployment Steps

### 1. Prepare Environment

```bash
export ENVIRONMENT=production
echo $DATABASE_URL
```

### Database Setup & Migration
```bash
# Run migrations
sqlx migrate run

# Verify migrations
sqlx migrate info
```

### Build Application
```bash
cargo build --release
cargo test
```

### Run Application
```bash
./target/release/stellar-tipjar-backend
```

### Production Service Setup
Create a service file:
```INI
[Unit]
Description=Stellar TipJar Backend
After=network.target

[Service]
ExecStart=/usr/local/bin/stellar-tipjar-backend
Restart=always
Environment=DATABASE_URL=your_db_url

[Install]
WantedBy=multi-user.target
```

Then:
```bash
systemctl daemon-reexec
systemctl enable tipjar-backend
systemctl start tipjar-backend
```

## Post-Deployment Verification
 - API responds correctly
 - Database connections working
 - No errors in logs
 - Stellar API reachable

## Rollback Procedure
```bash
# Stop service
systemctl stop tipjar-backend

# Restore previous binary
cp /backup/stellar-tipjar-backend /usr/local/bin/

# Revert migrations (if needed)
sqlx migrate revert

# Restart service
systemctl start tipjar-backend
```

## Deployment Notes
- Always test on staging before production
- Keep previous binaries for rollback
- Monitor logs after deployment




