# Runbooks — Common Operational Procedures

## Overview

This file provides step-by-step operational guides for frequent tasks.

---

## 1. Restart Service

```bash
systemctl restart tipjar-backend
systemctl status tipjar-backend
```

## 2. Check Service Logs
```bash
journalctl -u tipjar-backend -f
tail -f /var/log/tipjar/app.log
```

## 3. Database Health Check
```bash
psql $DATABASE_URL -c "SELECT COUNT(*) FROM creators;"
psql $DATABASE_URL -c "SELECT COUNT(*) FROM tips;"
```

## 4. Run Backup Test
```bash
./scripts/backup.sh --dry-run
```
- Verify backup integrity
- Test restore in staging environment

## 5. Performance Profiling
- CPU profiling:
```bash
cargo flamegraph
```
- Memory profiling:
```bash
heaptrack ./stellar-tipjar-backend
```

## 6. Deploy Patch
- Pull latest changes
- Build binary:
```bash
cargo build --release
```
- Stop service
```bash
systemctl stop tipjar-backend
```
- Replace binary
```bash
cp target/release/stellar-tipjar-backend /usr/local/bin/
```
- Start service
```bash
systemctl start tipjar-backend
```
- Verify service status



