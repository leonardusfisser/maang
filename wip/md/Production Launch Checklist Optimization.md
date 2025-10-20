# 🚀 Production Launch Checklist

## Pre-Launch Checklist

### Infrastructure (MP100 - Debian)

#### System Preparation
- [ ] Operating system updated: `sudo apt update && sudo apt upgrade`
- [ ] Firewall configured (ufw):
  ```bash
  sudo ufw allow 22/tcp   # SSH
  sudo ufw allow 80/tcp   # HTTP
  sudo ufw allow 443/tcp  # HTTPS
  sudo ufw enable
  ```
- [ ] Fail2ban installed for SSH protection
- [ ] Automatic security updates enabled
- [ ] Swap configured (if needed)
- [ ] NTP time synchronization enabled

#### User Setup
- [ ] Created `lillpepe` user: `sudo adduser lillpepe`
- [ ] Added to necessary groups: `sudo usermod -aG sudo lillpepe`
- [ ] SSH key authentication configured
- [ ] Password authentication disabled in SSH

#### Directory Structure
```bash
sudo mkdir -p /opt/lillpepe/{bin,templates,static,migrations,logs,backups,tenants}
sudo chown -R lillpepe:lillpepe /opt/lillpepe
sudo chmod 750 /opt/lillpepe
```

#### SurrealDB Installation
- [ ] SurrealDB installed: `curl -sSf https://install.surrealdb.com | sh`
- [ ] Systemd service created for SurrealDB
- [ ] SurrealDB configured to start on boot
- [ ] Database credentials secured (not default root/root)
- [ ] Backup directory: `/var/backups/surrealdb`

---

### Application Configuration

#### Environment Variables
- [ ] Created `/opt/lillpepe/.env` with production values
- [ ] File permissions: `chmod 600 /opt/lillpepe/.env`
- [ ] All sensitive data in .env (no hardcoded secrets)

```bash
# Production .env
HOST=0.0.0.0
PORT=8080
DATABASE_URL=127.0.0.1:8000
DATABASE_NAMESPACE=lillpepe
DATABASE_NAME=production
RUST_LOG=warn,lillpepe=info

# Stripe (PRODUCTION KEYS!)
STRIPE_API_KEY=sk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_ID=price_...

# Email (Production SMTP)
SMTP_HOST=smtp.eu.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@yourdomain.com
SMTP_PASSWORD=...
EMAIL_FROM=noreply@yourdomain.com
EMAIL_FROM_NAME=Lillpepe

# Alerts (optional)
ALERT_WEBHOOK_URL=https://hooks.slack.com/...
```

#### Systemd Service
- [ ] Service file created: `/etc/systemd/system/lillpepe.service`
- [ ] Service enabled: `sudo systemctl enable lillpepe`
- [ ] Service starts on boot
- [ ] Logs to journald
- [ ] Automatic restart on failure

---

### Security Hardening

#### SSL/TLS Certificates
- [ ] Domain DNS configured
- [ ] Certbot installed: `sudo apt install certbot python3-certbot-nginx`
- [ ] SSL certificates obtained: `sudo certbot certonly --standalone -d yourdomain.com`
- [ ] Auto-renewal configured: `sudo systemctl enable certbot.timer`
- [ ] HTTPS redirect configured in nginx

#### Nginx Reverse Proxy
```nginx
# /etc/nginx/sites-available/lillpepe
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;
    
    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    
    # SSL hardening
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### Application Security
- [ ] Admin passwords strong (16+ chars, mixed)
- [ ] Database passwords changed from defaults
- [ ] CSRF protection enabled
- [ ] Rate limiting configured
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention (parameterized queries ✓)
- [ ] XSS prevention (HTML escaping in templates ✓)

---

### Database

#### Migration & Setup
- [ ] Production database created
- [ ] Schema migrations run: `surreal import ...`
- [ ] Initial admin user created
- [ ] Database backed up before first deployment
- [ ] Connection pooling configured
- [ ] Indexes verified

#### Backup Strategy
- [ ] Automated daily backups: `lillpepe-admin backup`
- [ ] Backup cron job: `0 2 * * * /opt/lillpepe/bin/lillpepe-admin backup`
- [ ] Off-site backup copies (rsync to remote)
- [ ] Backup retention: 90 days
- [ ] Restore procedure tested

---

### Monitoring & Logging

#### Netdata Setup
```bash
bash <(curl -Ss https://my-netdata.io/kickstart.sh)
sudo systemctl enable netdata
sudo systemctl start netdata
```

- [ ] Netdata installed and running
- [ ] Accessible at: http://mp100:19999
- [ ] Alerts configured for:
    - CPU > 80%
    - Memory > 90%
    - Disk > 85%
    - Service down

#### Logging
- [ ] Application logs to journald
- [ ] Log rotation configured
- [ ] Log retention: 30 days
- [ ] Error alerts to email/webhook
- [ ] Failed job monitoring

#### Health Checks
- [ ] Health check endpoint: `/api/v1/health`
- [ ] Automated health checks: `*/5 * * * * /opt/lillpepe/bin/healthcheck`
- [ ] Uptime monitoring (external service)

---

### Stripe Configuration

#### Production Setup
- [ ] Live API keys configured (not test keys!)
- [ ] Webhook endpoint: `https://yourdomain.com/webhooks/stripe`
- [ ] Webhook signing secret configured
- [ ] Events subscribed:
    - invoice.payment_succeeded
    - invoice.payment_failed
    - customer.subscription.updated
    - customer.subscription.deleted
    - customer.subscription.created
- [ ] Webhook tested with Stripe CLI
- [ ] Products/prices created in Stripe Dashboard
- [ ] Payment methods enabled

---

### Email Configuration

#### Production SMTP
- [ ] Production SMTP credentials configured
- [ ] SPF record: `v=spf1 include:mailgun.org ~all`
- [ ] DKIM record configured
- [ ] DMARC record: `v=DMARC1; p=quarantine; rua=mailto:admin@yourdomain.com`
- [ ] Sender domain verified
- [ ] Email templates tested
- [ ] Bounce handling configured

---

## Deployment Process

### Initial Deployment

```bash
# On development machine
./deploy.fish setup     # First time only
./deploy.fish deploy    # Deploy application
```

### Verification Steps
1. **SSH to MP100**:
   ```bash
   ssh lillpepe@mp100
   ```

2. **Check service status**:
   ```bash
   sudo systemctl status lillpepe
   sudo systemctl status surrealdb
   sudo systemctl status nginx
   ```

3. **View logs**:
   ```bash
   sudo journalctl -u lillpepe -f --no-pager
   ```

4. **Test endpoints**:
   ```bash
   curl https://yourdomain.com/api/v1/health
   ```

5. **Admin login**:
    - Open https://yourdomain.com/admin
    - Login with admin credentials
    - Verify dashboard loads
    - Check all tabs work

---

## Post-Launch Monitoring

### Daily Checks (First Week)
- [ ] Service uptime: `sudo systemctl status lillpepe`
- [ ] Error logs: `sudo journalctl -u lillpepe -p err --since today`
- [ ] Failed jobs: Check admin dashboard
- [ ] Failed payments: Check payments tab
- [ ] Disk space: `df -h`
- [ ] Memory usage: `free -h`

### Weekly Checks
- [ ] Backup verification
- [ ] Security updates: `sudo apt update && sudo apt upgrade`
- [ ] SSL certificate expiry: `sudo certbot renew --dry-run`
- [ ] Database size: Check metrics
- [ ] Failed jobs cleanup

### Monthly Checks
- [ ] Review and archive old logs
- [ ] Database optimization
- [ ] Performance review
- [ ] Cost analysis (Stripe fees, hosting)
- [ ] User feedback review

---

## Performance Optimization

### Application Level

#### Database Queries
```rust
// Use indexes effectively
- [ ] Indexes on: domain, email, status, created_at
- [ ] Avoid N+1 queries
- [ ] Use SELECT * only when needed
- [ ] Batch operations where possible
```

#### Caching Strategy
```rust
// Implement caching for:
- [ ] Dashboard metrics (5 min cache)
- [ ] White label details (1 min cache)
- [ ] Static customization CSS (1 hour cache)
```

#### Connection Pooling
```rust
// Configure optimal pool sizes:
- [ ] Database connections: 10-20
- [ ] HTTP client: 10
- [ ] Background workers: 4-8
```

### System Level

#### Nginx Tuning
```nginx
# /etc/nginx/nginx.conf
worker_processes auto;
worker_connections 1024;

# Gzip compression
gzip on;
gzip_types text/css application/javascript application/json;

# Static file caching
location /static/ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

#### Systemd Resource Limits
```ini
# /etc/systemd/system/lillpepe.service
[Service]
LimitNOFILE=65536
MemoryMax=2G
CPUQuota=80%
```

---

## Scaling Strategy

### Current Capacity (Single Server)
- **White Labels**: 1000+
- **Concurrent Requests**: 100-200
- **API Throughput**: 500+ req/sec
- **Background Jobs**: ~100/minute

### When to Scale Up
- CPU consistently > 70%
- Memory consistently > 80%
- Response times > 200ms
- Failed job queue growing

### Scaling Options

#### Vertical (Easier)
- Upgrade MP100 resources
- Increase RAM: 16GB → 32GB
- Add CPU cores
- Faster SSD

#### Horizontal (Future)
1. **Database**:
    - SurrealDB cluster
    - Read replicas

2. **Application**:
    - Multiple app instances behind load balancer
    - Shared session store (Redis)

3. **Static Assets**:
    - CDN (Cloudflare)
    - Object storage (S3-compatible)

---

## Disaster Recovery

### Backup Locations
1. **Primary**: `/var/backups/lillpepe/`
2. **Off-site**: rsync to remote server
3. **Cold storage**: Weekly to external drive

### Recovery Procedures

#### Complete System Failure
1. Provision new server
2. Install dependencies
3. Restore latest backup:
   ```bash
   surreal import --conn http://localhost:8000 \
     /var/backups/lillpepe/backup_latest.tar.gz
   ```
4. Deploy application
5. Verify all services

#### Database Corruption
1. Stop lillpepe service
2. Restore from backup
3. Verify data integrity
4. Restart services

#### Lost Admin Access
1. SSH to server
2. Reset admin password:
   ```bash
   /opt/lillpepe/bin/lillpepe-admin reset-password \
     --email admin@yourdomain.com \
     --new-password new_secure_password
   ```

---

## Legal & Compliance

### GDPR Compliance
- [ ] Privacy Policy published at `/legal/privacy`
- [ ] Terms of Service at `/legal/terms`
- [ ] Cookie consent (if using cookies)
- [ ] Data processing agreement with Stripe
- [ ] User data export functionality
- [ ] User data deletion functionality
- [ ] Breach notification procedure

### Financial Compliance
- [ ] Invoice retention: 7 years
- [ ] VAT handling (if EU customers)
- [ ] Payment records secure
- [ ] Stripe compliance verified

---

## Support & Maintenance

### Customer Support
- [ ] Support email: support@yourdomain.com
- [ ] Response time SLA: 24-48 hours
- [ ] Documentation site (optional)
- [ ] FAQ page

### Maintenance Windows
- **Planned**: Sunday 2-4 AM (announce 1 week ahead)
- **Emergency**: As needed (notify immediately)
- **Updates**: Weekly security patches

---

## Final Pre-Launch Checklist

### Critical Path
- [ ] All tests passing: `cargo test --all`
- [ ] Admin login works
- [ ] Can create white label
- [ ] Can provision white label
- [ ] Stripe webhooks working
- [ ] Emails sending
- [ ] Health checks passing
- [ ] Backups automated
- [ ] SSL certificates valid
- [ ] Monitoring active

### Nice to Have
- [ ] Load testing completed
- [ ] Documentation updated
- [ ] Blog post drafted
- [ ] Social media ready
- [ ] Support processes documented

---

## Launch Day Checklist

### T-1 Hour
- [ ] Fresh backup created
- [ ] All services confirmed running
- [ ] Health checks passing
- [ ] Monitoring dashboard open

### T-0 (Launch!)
- [ ] DNS switched to production
- [ ] SSL verified on production domain
- [ ] First real signup tested
- [ ] Payment flow tested
- [ ] Admin dashboard confirmed working

### T+1 Hour
- [ ] Monitor error logs
- [ ] Check Stripe dashboard
- [ ] Verify emails sending
- [ ] Watch resource usage

### T+24 Hours
- [ ] Review all logs
- [ ] Check for any errors
- [ ] Verify backups running
- [ ] Monitor customer signups

---

## Emergency Contacts

```
Developer: your@email.com
Hosting: MP100 SSH access
Database: SurrealDB docs
Stripe: dashboard.stripe.com/support
Email: Mailgun support
Domain: Your registrar support
```

---

## Success Metrics

### Week 1 Goals
- [ ] 0 critical errors
- [ ] < 100ms average response time
- [ ] 100% uptime
- [ ] 5+ white label signups

### Month 1 Goals
- [ ] 50+ active white labels
- [ ] €500+ MRR
- [ ] < 1% payment failure rate
- [ ] 99.9% uptime

---

**🎉 You're ready to launch!**

This checklist covers everything needed for a production-ready deployment. Remember:
- Start small, monitor closely
- Fix issues as they appear
- Scale gradually
- Keep backups current
- Document everything

**Good luck! 🚀**