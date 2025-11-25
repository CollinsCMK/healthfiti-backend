#!/bin/bash
set -a
source .env
set +a

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo ".env file not found! Create one before running."
    exit 1
fi

# Variables
HOME_DIR="/home/$PROJECT_USER"
MINIO_DIR="$HOME_DIR/minio-data"
MINIO_BIN="$HOME_DIR/go/bin/minio"
SERVICE_FILE="/etc/systemd/system/minio.service"

echo "-------------------------------------------"
echo "Using MinIO Settings:"
echo "Run User:      $PROJECT_USER"
echo "Access Key:    $MINIO_ACCESS_KEY"
echo "Secret Key:    $MINIO_SECRET_KEY"
echo "-------------------------------------------"

# Create project user if missing
if id "$PROJECT_USER" >/dev/null 2>&1; then
    echo "User $PROJECT_USER exists."
else
    echo "Creating user $PROJECT_USER..."
    sudo useradd -m -s /bin/bash "$PROJECT_USER"
    echo "User $PROJECT_USER created."
fi

# Update system
sudo apt update -y
sudo apt install -y wget nano

# Ensure home directory exists & owned correctly
sudo mkdir -p "$HOME_DIR"
sudo chown -R "$PROJECT_USER:$PROJECT_USER" "$HOME_DIR"

# Create MinIO data directory
sudo mkdir -p "$MINIO_DIR"
sudo chown -R "$PROJECT_USER:$PROJECT_USER" "$MINIO_DIR"

# Install Go if missing
if ! command -v go &> /dev/null; then
    echo "Installing Go..."
    wget https://go.dev/dl/go1.22.10.linux-amd64.tar.gz -O /tmp/go.tar.gz
    sudo rm -rf /usr/local/go
    sudo tar -C /usr/local -xzf /tmp/go.tar.gz
    echo "export PATH=\$PATH:/usr/local/go/bin" >> /home/"$PROJECT_USER"/.profile
    export PATH=$PATH:/usr/local/go/bin
fi

# Install MinIO binary as the project user
echo "Installing MinIO..."
sudo -u "$PROJECT_USER" /usr/local/go/bin/go install github.com/minio/minio@latest

# Create systemd service file
echo "Creating systemd service..."
sudo tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=MinIO Object Storage
After=network.target

[Service]
User=$PROJECT_USER
Environment=MINIO_ROOT_USER=$MINIO_ACCESS_KEY
Environment=MINIO_ROOT_PASSWORD=$MINIO_SECRET_KEY
ExecStart=$MINIO_BIN server $MINIO_DIR --console-address ":9001"
Restart=always
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Enable & start MinIO
sudo systemctl daemon-reload
sudo systemctl enable minio
sudo systemctl restart minio

echo "-------------------------------------------"
echo "MinIO Installed and Running!"
echo "S3 Endpoint:  http://127.0.0.1:9000"
echo "Console:      http://127.0.0.1:9001"
echo "Username:     $MINIO_ACCESS_KEY"
echo "Password:     $MINIO_SECRET_KEY"
echo "-------------------------------------------"
echo "View logs with:"
echo "  sudo journalctl -u minio -f"
echo "-------------------------------------------"
