# Docker

```yml
version: "3"

services:
  mikan-proxy:
    image: ghcr.io/kotahv/mikan-proxy:latest
    container_name: mikan-proxy
    environment:
      - MIKAN_URL=https://<yourdomain>
    restart: unless-stopped
    ports:
      - <port>:80
```