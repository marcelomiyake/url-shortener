FROM node:24.21.0-bookworm-slim AS frontend-build
WORKDIR /workspace/frontend
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts
COPY . ./
RUN npm run build

FROM nginxinc/nginx-unprivileged:1.30.5-alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=frontend-build /workspace/frontend/dist /usr/share/nginx/html
EXPOSE 8080
