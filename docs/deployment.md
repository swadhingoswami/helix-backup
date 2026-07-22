# HELIX Deployment Guide

## Production Deployment

### Prerequisites

- **Linux**: kernel 5.4+, `dm-era` kernel module, minimum 256 MB RAM
- **macOS**: macOS 11+, minimum 512 MB RAM
- **Storage**: Backup repository requires adequate space (see sizing below)
- **Permissions**: Root/sudo access for raw block device access

### Installation

#### From Pre-built Binaries

```bash
# Download from GitHub Releases
curl -LO https://github.com/helix/helix/releases/latest/download/helix-linux-amd64.tar.gz
tar xzf helix-linux-amd64.tar.gz
sudo mv helix /usr/local/bin/helix
```

#### From Source

```bash
cargo install helix
```

### Repository Sizing

| Backup Type | Storage Required | Notes |
|---|---|---|
| Full (1 TB) | ~1 TB | Full copy of all blocks |
| Incremental (daily, 1% change) | ~10 GB/day | Only changed blocks |
| Metadata | ~1 GB per 100 TB | Manifests + SQLite index |
| Temporary workspace | ~10 GB | During backup operations |

**Rule of thumb**: Allocate 2x the source data size for the repository to accommodate operations.

### Configuration

Create `/etc/helix/config.yaml`:

```yaml
block_size: 4096
storage:
  repository_path: /var/helix/backups
  max_parallel_io: 4
backup:
  retention_days: 30
  verify_after_backup: true
logging:
  level: info
  file: /var/log/helix/helix.log
```

### Systemd Service (Linux)

```ini
# /etc/systemd/system/helix-backup.service
[Unit]
Description=HELIX Block-Level Backup
Documentation=https://helix-backup.io/docs
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/helix incremental /dev/sda --dest /var/helix/backups
User=root
Group=root
Environment=RUST_LOG=info
Nice=10
IOSchedulingClass=best-effort
IOSchedulingPriority=7

[Install]
WantedBy=multi-user.target
```

### Systemd Timer (Scheduled Backups)

```ini
# /etc/systemd/system/helix-backup.timer
[Unit]
Description=Daily HELIX Backup
Requires=helix-backup.service

[Timer]
OnCalendar=daily
Persistent=true
RandomizedDelaySec=1800

[Install]
WantedBy=timers.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable helix-backup.timer
sudo systemctl start helix-backup.timer
```

### Cron-based Scheduling

```bash
# /etc/cron.d/helix-backup
0 2 * * * root /usr/local/bin/helix incremental /dev/sda --dest /var/helix/backups --label "daily-$(date +\%Y\%m\%d)"
```

### Monitoring

#### Prometheus Metrics

HELIX exposes operational metrics that can be integrated with monitoring systems:

```bash
# Check repository status
helix check /var/helix/backups

# List backups for monitoring
helix list /var/helix/backups --json
```

#### Log Monitoring

```bash
# View backup logs
tail -f /var/log/helix/helix.log

# Check for errors
grep ERROR /var/log/helix/helix.log
```

### Disaster Recovery

#### Backup of Backup Metadata

The repository metadata (`metadata.json` and `index.db`) is critical for recovery. Back up these files independently:

```bash
# Backup metadata separately
cp /var/helix/backups/metadata.json /backup/metadata-backup/
cp /var/helix/backups/index.db /backup/metadata-backup/
```

#### Recovery from Metadata Loss

If repository metadata is lost but block data remains:

```bash
# Attempt auto-recovery
helix check /var/helix/backups --repair
```

### Security Hardening

1. **File permissions**:
   ```bash
   chmod 600 /etc/helix/config.yaml
   chmod 600 /etc/helix/backup.key
   chmod 700 /var/helix/backups
   ```

2. **Run as dedicated user**:
   ```bash
   useradd -r -s /bin/false helix-backup
   chown helix-backup: /var/helix/backups
   ```

3. **Encryption at rest**: Enable encryption in config

4. **Network isolation**: Keep backup repository on isolated storage

### Performance Tuning

#### Linux I/O Scheduler

```bash
# Use noop scheduler for SSDs
echo noop > /sys/block/sda/queue/scheduler

# Increase read-ahead
blockdev --setra 4096 /dev/sda
```

#### Kernel Parameters

```bash
# Increase I/O limits
sysctl -w vm.dirty_ratio=10
sysctl -w vm.dirty_background_ratio=5
sysctl -w vm.vfs_cache_pressure=200
```

### Backup Verification Strategy

1. **Automated**: `verify_after_backup: true` in config
2. **Scheduled**: Periodic `helix check` via cron
3. **Restore test**: Full restore to test environment quarterly

### Upgrade Procedure

```bash
# 1. Download new version
# 2. Run repository check
helix check /var/helix/backups

# 3. Create a full backup as safety
helix full /dev/sda --dest /var/helix/backups --label "pre-upgrade-backup"

# 4. Replace binary
sudo cp helix /usr/local/bin/helix

# 5. Verify new version
helix --version

# 6. Run test backup
helix incremental /dev/sda --dest /var/helix/backups --label "post-upgrade-test"
```
