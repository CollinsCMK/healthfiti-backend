#!/bin/bash
set -a
source .env
set +a

# Usage:
#   ./sea_orm.sh refresh   # Drop all and reapply migrations, then regenerate entities
#   ./sea_orm.sh up        # Apply pending migrations, then regenerate entities
#   ./sea_orm.sh fresh     # Drop all tables (fresh) and reapply migrations, then regen entities

# Load environment variables from .env
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Config paths
ENTITY_MAIN_OUTPUT_DIR="entity-main/src"
ENTITY_TENANT_OUTPUT_DIR="entity-tenant/src"
MIGRATION_MAIN_DIR="migration-main"
MIGRATION_TENANT_DIR="migration-tenant"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

show_usage() {
    echo -e "${YELLOW}Usage: $0 {refresh|up|fresh}${NC}"
    exit 1
}

case "$1" in
    refresh|up|fresh) ;;
    *) show_usage ;;
esac

if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}DATABASE_URL missing in .env.${NC}"
    exit 1
fi

if [ -z "$TENANT_DATABASE_URL" ]; then
    echo -e "${RED}TENANT_DATABASE_URL missing in .env.${NC}"
    exit 1
fi

if ! command -v sea-orm-cli &> /dev/null; then
    echo -e "${RED}Install sea-orm-cli first:${NC}"
    echo "cargo install sea-orm-cli@^2.0.0-rc"
    exit 1
fi

#############################
# MAIN DB MIGRATIONS
#############################
if [ "$1" == "refresh" ]; then
    echo -e "${YELLOW}🔄 Refreshing MAIN DB migrations...${NC}"
    sea-orm-cli migrate refresh -d $MIGRATION_MAIN_DIR
elif [ "$1" == "fresh" ]; then
    echo -e "${YELLOW}🧨 Running FRESH MAIN DB migrations...${NC}"
    sea-orm-cli migrate fresh -d $MIGRATION_MAIN_DIR
else
    echo -e "${YELLOW}⬆ Applying MAIN DB migrations...${NC}"
    sea-orm-cli migrate up -d $MIGRATION_MAIN_DIR
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}Main DB migration failed.${NC}"
    exit 1
fi

#############################
# TENANT DB MIGRATIONS
#############################
if [ "$1" == "refresh" ]; then
    echo -e "${YELLOW}🔄 Refreshing TENANT DB migrations...${NC}"
    sea-orm-cli migrate refresh -d $MIGRATION_TENANT_DIR --database-url $TENANT_DATABASE_URL
elif [ "$1" == "fresh" ]; then
    echo -e "${YELLOW}🧨 Running FRESH TENANT DB migrations...${NC}"
    sea-orm-cli migrate fresh -d $MIGRATION_TENANT_DIR --database-url $TENANT_DATABASE_URL
else
    echo -e "${YELLOW}⬆ Applying TENANT DB migrations...${NC}"
    sea-orm-cli migrate up -d $MIGRATION_TENANT_DIR --database-url $TENANT_DATABASE_URL
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}Tenant DB migration failed.${NC}"
    exit 1
fi

#############################
# MAIN ENTITY GENERATION
#############################
echo -e "${YELLOW}📦 Generating MAIN DB entities...${NC}"
sea-orm-cli generate entity \
    --with-serde both \
    --database-url $DATABASE_URL \
    --output-dir $ENTITY_MAIN_OUTPUT_DIR \
    --entity-format dense

if [ $? -ne 0 ]; then
    echo -e "${RED}Main entity generation failed.${NC}"
    exit 1
fi

#############################
# TENANT ENTITY GENERATION
#############################
echo -e "${YELLOW}📦 Generating TENANT DB entities...${NC}"
sea-orm-cli generate entity \
    --with-serde both \
    --database-url $TENANT_DATABASE_URL \
    --output-dir $ENTITY_TENANT_OUTPUT_DIR \
    --entity-format dense

if [ $? -ne 0 ]; then
    echo -e "${RED}Tenant entity generation failed.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All migrations + entity generation completed successfully!${NC}"
