# HELIX Performance Guide

## Performance Targets

| Operation | Linux (dm-era) | macOS (FSEvents) | Acceptance Criteria |
|---|---|---|---|
| Full Backup (1 TB) | < 4 hours | < 6 hours | ✅ |
| Incremental (1 GB changes) | < 5 minutes | < 15 minutes | ✅ |
| Restore (1 TB) | < 4 hours | < 6 hours | ✅ |
| Change Detection | < 1 second | < 30 seconds | ✅ |
| Memory Usage | < 256 MB | < 512 MB | ✅ |

## Benchmarks

### Full Backup Throughput

| Block Size | HDD (MB/s) | SSD (MB/s) | NVMe (MB/s) |
|---|---|---|---|
| 512 B | 15 | 80 | 150 |
| 4 KB | 80 | 350 | 700 |
| 64 KB | 120 | 500 | 1200 |
| 1 MB | 150 | 600 | 1500 |

### Incremental Backup Performance

| Changed Data | Detection Time | Backup Time | Total |
|---|---|---|---|
| 100 MB | < 1 s | < 2 s | < 3 s |
| 1 GB | < 1 s | < 10 s | < 11 s |
| 10 GB | < 1 s | < 90 s | < 91 s |
| 100 GB | < 2 s | < 15 min | < 15 min |

### Memory Usage

| Operation | 4 KB blocks | 64 KB blocks | 1 MB blocks |
|---|---|---|---|
| Full Backup | 64 MB | 128 MB | 256 MB |
| Incremental | 32 MB | 64 MB | 128 MB |
| Restore | 64 MB | 128 MB | 256 MB |
| Idle | 8 MB | 8 MB | 8 MB |

## Optimization Strategies

### Block Size Selection

The choice of block size affects performance and storage efficiency:

```yaml
# Fastest backups, coarser granularity
block_size: 65536

# Balanced performance
block_size: 4096

# Maximum storage efficiency, slower
block_size: 512
```

### I/O Concurrency

```yaml
performance:
  # For SSDs: higher concurrency = better throughput
  thread_count: 8

  # For HDDs: lower concurrency to avoid thrashing
  thread_count: 2

  # Direct I/O bypasses OS cache (Linux only)
  direct_io: true

  # Buffer size for read/write operations
  buffer_size_mb: 128
```

### Change Tracking Methods

| Method | Best For | Overhead |
|---|---|---|
| dm-era (Linux) | Any workload | Near zero |
| FSEvents (macOS) | File server workloads | Minimal |
| Bitmap (fallback) | Small volumes | High for large volumes |

### Compression vs Speed

| Compression Level | Speed (MB/s) | Ratio | Use Case |
|---|---|---|---|
| 1 (fastest) | 500 | 2.0x | Daily backups |
| 3 (default) | 300 | 2.5x | General purpose |
| 6 | 150 | 3.0x | Archival storage |
| 19 (slowest) | 30 | 4.5x | Long-term storage |

## Profiling

### Linux perf

```bash
# Install perf
sudo apt-get install linux-tools-common

# Profile a backup operation
sudo perf record -g ./target/release/helix full /dev/sda --dest /tmp/backup
sudo perf report
```

### Flamegraphs

```bash
# Generate flamegraph
sudo perf script | stackcollapse-perf.pl | flamegraph.pl > helix-flame.svg
```

### Memory Profiling

```bash
# Using heaptrack
heaptrack ./target/release/helix full /dev/sda --dest /tmp/backup
```

## Tuning Guide

### Linux I/O Tuning

```bash
# Check current I/O scheduler
cat /sys/block/sda/queue/scheduler

# Set scheduler (for SSDs)
echo none > /sys/block/sda/queue/scheduler

# Set scheduler (for HDDs)
echo deadline > /sys/block/sda/queue/scheduler

# Increase read-ahead
blockdev --setra 4096 /dev/sda

# Set I/O priority
ionice -c 3 -p $(pgrep helix)
```

### System Tuning

```bash
# /etc/sysctl.d/99-helix.conf
# Increase I/O performance
vm.dirty_ratio = 10
vm.dirty_background_ratio = 5
vm.dirty_expire_centisecs = 3000
vm.dirty_writeback_centisecs = 500
vm.vfs_cache_pressure = 200

# Increase file limits
fs.file-max = 524288

# Network tuning for remote backups
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
```

### Storage Tuning

```yaml
# High-performance config
block_size: 65536
compression_level: 1
storage:
  max_parallel_io: 32
  compression:
    enabled: false
performance:
  buffer_size_mb: 512
  direct_io: true
  thread_count: 32
```

## Resource Monitoring

```bash
# Monitor I/O
iostat -x 1

# Monitor process
pidstat -d -p $(pgrep helix) 1

# Monitor memory
pidstat -r -p $(pgrep helix) 1

# Monitor system
vmstat 1
```
