# Incident Response Playbook

## Overview

This document outlines the procedures for responding to incidents affecting the Stellar Tip Jar backend. It ensures consistent handling of outages, performance issues, and security events.

---

## Severity Levels

### SEV1 — Critical
- Complete service outage
- Data loss or corruption
- Security breach

**Response Time:** Immediate

---

### SEV2 — High
- Partial service degradation
- Error rate > 5%
- Significant performance issues

**Response Time:** Within 15 minutes

---

### SEV3 — Medium
- Minor feature failures
- Non-critical issues

**Response Time:** Within 1 hour

---

## Incident Response Process

### 1. Detection and Triage

- Incident detected via:
  - Monitoring alerts
  - User reports
  - Logs

- Determine severity level
- Assign on-call engineer

---

### 2. Initial Response

- Acknowledge the incident
- Confirm issue is reproducible
- Notify team (Slack/Discord/email)
- Create incident tracking issue (GitHub)

---

### 3. Investigation

- Check logs:
  ```bash
  journalctl -u tipjar-backend -n 100
  ```
- Verify service status:
```bash
systemctl status tipjar-backend
```
- Check database connectivity:
```bash
psql $DATABASE_URL -c "SELECT 1"
```
- Test Stellar Horizon API:
```bash
curl https://horizon-testnet.stellar.org/
```

### 4. Mitigation
Depending on root cause:
- Restart service:
```bash
systemctl restart tipjar-backend
```
- Rollback deployment (if recent change caused issue)
- Fix configuration or environment variables
- Reconnect to database or external services

### 5. Recovery
- Verify system is stable
- Confirm API endpoints are responding
- Ensure no errors in logs
- Monitor for recurrence

### 6. Communication
- Provide status updates:
 - Issue identified
 - Fix in progress
 - Resolved
- Notify stakeholders when resolved

### 7. Post-Incident
- Document the incident:
 - Root cause
 - Impact
 - Timeline
 - Resolution steps
- Identify improvements
- Implement preventive fixes

## Communication Templates

### Incident Acknowledgment
```text
We are currently investigating an issue affecting the Stellar Tip Jar backend. Our team is actively working on a fix.
```

### Incident Resolved
```text
The issue has been resolved. Services are now operating normally. We will continue monitoring to ensure stability.
```

## Post-Mortem Template
- Incident Summary
- Severity Level
- Start Time / End Time
- Root Cause
- Impact
- Resolution
- Action Items

## On-Call Responsibilities
- Monitor alerts
- Respond within SLA time
- Escalate when necessary
- Document incidents properly




