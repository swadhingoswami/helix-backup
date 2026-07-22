#!/usr/bin/env bash
# =============================================================================
# HELIX - Basic Usage Examples
# =============================================================================
# This script demonstrates common HELIX operations.
# Run with: bash examples/basic_usage.sh
# =============================================================================

set -euo pipefail

REPO="/tmp/helix-demo-repo"
DEVICE=${1:-"/dev/loop0"}
BOLD='\033[1m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

print_step() {
    echo -e "\n${BOLD}${GREEN}=== $1 ===${NC}\n"
}

cleanup() {
    echo "Cleaning up..."
    rm -rf "$REPO"
    if [ -f /tmp/helix-demo.img ]; then
        sudo losetup -d "$DEVICE" 2>/dev/null || true
        rm -f /tmp/helix-demo.img
    fi
}
trap cleanup EXIT

# Step 1: Initialize a backup repository
print_step "1. Initializing Backup Repository"
helix init "$REPO"
echo "Repository created at: $REPO"

# Step 2: Create a test device (loop device)
print_step "2. Creating Test Block Device"
dd if=/dev/zero of=/tmp/helix-demo.img bs=1M count=100 2>/dev/null
sudo losetup "$DEVICE" /tmp/helix-demo.img
echo "Test device created: $DEVICE"

# Step 3: Perform a full backup
print_step "3. Performing Full Backup"
helix full "$DEVICE" --dest "$REPO" --label "initial-backup"
echo "Full backup complete"

# Step 4: List backups
print_step "4. Listing Backups"
helix list "$REPO"
echo ""

# Step 5: Perform an incremental backup
print_step "5. Performing Incremental Backup"
# Write some data to the device
echo "New data" | sudo dd of="$DEVICE" bs=4096 seek=10 2>/dev/null
helix incremental "$DEVICE" --dest "$REPO" --label "incremental-001"
echo "Incremental backup complete"

# Step 6: List backups again
print_step "6. Listing Backups After Incremental"
helix list "$REPO"

# Step 7: Validate repository
print_step "7. Validating Repository"
helix check "$REPO"
echo "Repository integrity verified"

# Step 8: Show configuration
print_step "8. Current Configuration"
helix config show

echo -e "\n${BOLD}${GREEN}All basic operations completed successfully!${NC}"
echo "Repository at: $REPO"
echo "Device: $DEVICE"
echo -e "\nTry these commands yourself:"
echo "  helix list --json $REPO"
echo "  helix check $REPO"
echo "  helix restore $REPO /tmp/restored-device --point latest"
