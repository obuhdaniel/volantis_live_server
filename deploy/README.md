# Production Deployment Guide

This directory contains the LiveKit server configuration for `livekit-server.volantislive.com` and `livekit-turn.volantislive.com`.

## Structure

- `init_script.sh` - Full installation script for Linux VMs
- `docker-compose.yaml` - Docker Compose for manual deployment
- `livekit.yaml` - LiveKit server configuration
- `caddy.yaml` - Caddy L4 proxy configuration
- `redis.conf` - Redis configuration

## Prerequisites

1. A Linux VM (Ubuntu, Amazon Linux, or similar)
2. DNS records pointing to your VM:
   - `livekit-server.volantislive.com` -> VM IP
   - `livekit-turn.volantislive.com` -> VM IP
3. Ports open on firewall (see LiveKit docs): 443, 80, 7881, 3478/UDP, 50000-60000/UDP

## Deployment Options

### Option 1: Cloud Init (Recommended for AWS/Azure/DigitalOcean)

1. Launch a new VM with your cloud provider
2. Paste contents of `init_script.sh` into the "User data" field
3. The VM will auto-configure on startup

### Option 2: Manual Deployment

1. SSH into your VM
2. Copy `init_script.sh` to the VM
3. Run: `sudo ./init_script.sh`

### Option 3: Docker Compose (for testing)

```bash
cd deploy/livekit-server.volantislive.com
docker-compose up -d
```

## Application Deployment

The Rust app (`lk-app`) connects to the production LiveKit server at:
- `wss://livekit-server.volantislive.com`

Update environment variables in docker-compose.yml:
```yaml
LIVEKIT_URL: wss://livekit-server.volantislive.com
LIVEKIT_API_KEY: APIY7KcSayArEyo
LIVEKIT_API_SECRET: zewvscS0R9ieMngmHjf1ZuwHcmJeznzNeiC1wb556msA
```

## Upgrading LiveKit

Edit `docker-compose.yaml` and change the image tag:
```yaml
livekit:
  image: livekit/livekit-server:v1.x.x
```

Then:
```bash
cd /opt/livekit
docker-compose pull
docker-compose up -d
```

## Verification

Check LiveKit server status:
```bash
systemctl status livekit-docker
docker-compose logs -f
```

Check TLS certificates (look for "certificate obtained successfully"):
```bash
docker logs livekit-caddy-1 2>&1 | grep -i certificate
```