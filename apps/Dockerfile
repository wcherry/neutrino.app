# ---------- Build Stage ----------
FROM node:20-alpine AS builder

WORKDIR /app

# Enable pnpm
RUN corepack enable

# Copy workspace manifests first (for caching)
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY .npmrc ./
COPY web/package.json web/
COPY packages/*/package.json packages/*/

# Install dependencies
RUN pnpm install --prod=false

# Copy the rest of the repository
COPY . .

# Build the web app
WORKDIR /app/web
RUN pnpm build
# RUN pnpm export

# ---------- Runtime Stage ----------
FROM nginx:alpine

RUN rm /etc/nginx/conf.d/default.conf

COPY neutrino-proxy.conf /etc/nginx/conf.d/default.conf

# Copy static output
COPY --from=builder /app/web/out /usr/share/nginx/html

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
