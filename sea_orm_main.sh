#!/bin/bash
set -a
source .env
set +a

# Usage:
#   ./sea_orm_main.sh refresh   # Drop all and reapply migrations, then regenerate entities
#   ./sea_orm_main.sh up        # Apply pending migrations, then regenerate entities

# Load environment variables from .env file if present
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Configuration
ENTITY_OUTPUT_DIR="entity-main/src"
MIGRATION_OUTPUT_DIR="migration-main"

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Show help
show_usage() {
    echo -e "${YELLOW}Invalid or missing command.${NC}"
    echo -e "${YELLOW}Usage: $0 {refresh|up}${NC}"
    echo -e "${YELLOW}  refresh${NC} - Drops all and reapplies migrations, then regenerates entities"
    echo -e "${YELLOW}  up     ${NC} - Applies pending migrations, then regenerates entities"
    exit 1
}

# Ensure valid argument
case "$1" in
    refresh|up)
        ;;
    *)
        show_usage
        ;;
esac

# Check for DATABASE_URL
if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}DATABASE_URL is not set. Define it in your .env file.${NC}"
    exit 1
fi

# Check for sea-orm-cli
if ! command -v sea-orm-cli &> /dev/null; then
    echo -e "${RED}sea-orm-cli is not installed. Install it with:${NC}"
    echo -e "${YELLOW}cargo install sea-orm-cli${NC}"
    exit 1
fi

# Run migrations
if [ "$1" == "refresh" ]; then
    echo "Refreshing migrations (drop and reapply)..."
    sea-orm-cli migrate refresh -d $MIGRATION_OUTPUT_DIR
else
    echo "Running pending migrations..."
    sea-orm-cli migrate up -d $MIGRATION_OUTPUT_DIR
fi

# Exit if migration failed
if [ $? -ne 0 ]; then
    echo -e "${RED}Migration failed. Check error above.${NC}"
    exit 1
fi

# Generate entities with serde
echo "Generating SeaORM entities with serde..."
sea-orm-cli generate entity --with-serde both --database-url $DATABASE_URL --output-dir $ENTITY_OUTPUT_DIR --entity-format dense

if [ $? -ne 0 ]; then
    echo -e "${RED}Entity generation failed.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Migration and entity generation completed successfully.${NC}"