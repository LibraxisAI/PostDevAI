#!/bin/bash
# Setup RAM disk for PostDevAI Dragon Node

# Default settings
RAMDISK_SIZE_GB=200
RAMDISK_PATH="/mnt/ramlake"
BACKUP_PATH="/var/backups/ramlake"

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}     PostDevAI RAM-Lake Setup Tool     ${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --size=*)
      RAMDISK_SIZE_GB="${1#*=}"
      shift
      ;;
    --path=*)
      RAMDISK_PATH="${1#*=}"
      shift
      ;;
    --backup=*)
      BACKUP_PATH="${1#*=}"
      shift
      ;;
    --help)
      echo "Usage: setup_ramdisk.sh [options]"
      echo
      echo "Options:"
      echo "  --size=SIZE    RAM disk size in GB (default: 200)"
      echo "  --path=PATH    Mount point for RAM disk (default: /mnt/ramlake)"
      echo "  --backup=PATH  Backup directory (default: /var/backups/ramlake)"
      echo "  --help         Show this help message"
      echo
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# Check if running as root
if [ "$(id -u)" -ne 0 ]; then
  echo -e "${RED}Error: This script must be run as root${NC}"
  echo "Please try again with 'sudo $0 $*'"
  exit 1
fi

# Verify macOS
if [ "$(uname)" != "Darwin" ]; then
  echo -e "${RED}Error: This script is only intended for macOS${NC}"
  exit 1
fi

echo -e "${BLUE}Configuration:${NC}"
echo -e "${BLUE}  RAM disk size: ${YELLOW}${RAMDISK_SIZE_GB} GB${NC}"
echo -e "${BLUE}  Mount point:   ${YELLOW}${RAMDISK_PATH}${NC}"
echo -e "${BLUE}  Backup path:   ${YELLOW}${BACKUP_PATH}${NC}"
echo

# Convert GB to sectors (512-byte blocks)
# 1 GB = 1024 MB = 1024*1024 KB = 1024*1024*2 sectors
SECTORS=$((RAMDISK_SIZE_GB * 1024 * 1024 * 2))
echo -e "${BLUE}Converting ${RAMDISK_SIZE_GB} GB to ${SECTORS} sectors${NC}"

# Check if mount point exists
if [ ! -d "$RAMDISK_PATH" ]; then
  echo -e "${BLUE}Creating mount point: ${YELLOW}${RAMDISK_PATH}${NC}"
  mkdir -p "$RAMDISK_PATH"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to create mount point${NC}"
    exit 1
  fi
fi

# Check if backup path exists
if [ ! -d "$BACKUP_PATH" ]; then
  echo -e "${BLUE}Creating backup directory: ${YELLOW}${BACKUP_PATH}${NC}"
  mkdir -p "$BACKUP_PATH"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to create backup directory${NC}"
    exit 1
  fi
fi

# Check if a RAM disk is already mounted
if mount | grep -q "$RAMDISK_PATH"; then
  echo -e "${YELLOW}Warning: A volume is already mounted at ${RAMDISK_PATH}${NC}"
  echo -n "Do you want to unmount it? [y/N] "
  read -r UNMOUNT
  if [[ "$UNMOUNT" =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Unmounting existing volume...${NC}"
    diskutil unmount "$RAMDISK_PATH"
    if [ $? -ne 0 ]; then
      echo -e "${RED}Error: Failed to unmount existing volume${NC}"
      exit 1
    fi
  else
    echo -e "${RED}Aborted by user${NC}"
    exit 1
  fi
fi

# Create and mount RAM disk
echo -e "${BLUE}Creating and mounting RAM disk...${NC}"
DEV=$(hdiutil attach -nomount ram://${SECTORS})
if [ $? -ne 0 ]; then
  echo -e "${RED}Error: Failed to create RAM disk${NC}"
  exit 1
fi

echo -e "${BLUE}Formatting RAM disk as HFS+...${NC}"
diskutil erasevolume HFS+ "RAM-Lake" "$DEV"
if [ $? -ne 0 ]; then
  echo -e "${RED}Error: Failed to format RAM disk${NC}"
  hdiutil detach "$DEV"
  exit 1
fi

# Verify mount point
if ! mount | grep -q "$DEV"; then
  echo -e "${RED}Error: RAM disk not mounted properly${NC}"
  exit 1
fi

# Create directory structure
echo -e "${BLUE}Creating RAM-Lake directory structure...${NC}"
mkdir -p "$RAMDISK_PATH/vectors"
mkdir -p "$RAMDISK_PATH/code"
mkdir -p "$RAMDISK_PATH/history"
mkdir -p "$RAMDISK_PATH/metadata"

# Set permissions
echo -e "${BLUE}Setting permissions...${NC}"
chmod 755 "$RAMDISK_PATH"
chmod 755 "$RAMDISK_PATH/vectors"
chmod 755 "$RAMDISK_PATH/code"
chmod 755 "$RAMDISK_PATH/history"
chmod 755 "$RAMDISK_PATH/metadata"

# Success message
echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}     RAM-Lake Setup Successful!        ${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "RAM disk mounted at: ${YELLOW}${RAMDISK_PATH}${NC}"
echo -e "Device:             ${YELLOW}${DEV}${NC}"
echo -e "Size:               ${YELLOW}${RAMDISK_SIZE_GB} GB${NC}"
echo
echo -e "${YELLOW}Note: This RAM disk will be lost on system reboot.${NC}"
echo -e "${YELLOW}To create it automatically at startup, consider adding${NC}"
echo -e "${YELLOW}this script to your startup items or creating a launchd job.${NC}"
echo

# Save the device information for later use
echo "$DEV" > "$RAMDISK_PATH/.device_info"

exit 0