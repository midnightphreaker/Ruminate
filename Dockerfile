FROM node:22-alpine AS build

WORKDIR /app
COPY package*.json ./
RUN npm install --ignore-scripts
COPY tsconfig.json ./
COPY index.ts lib.ts ./
RUN npm run build && npm prune --omit=dev

FROM node:22-alpine

ENV NODE_ENV=production
ENV PORT=8000
WORKDIR /app
RUN addgroup -g 10001 -S app && adduser -S app -u 10001 -G app
COPY --from=build --chown=app:app /app/package*.json ./
COPY --from=build --chown=app:app /app/node_modules ./node_modules
COPY --from=build --chown=app:app /app/dist ./dist
USER app
EXPOSE 8000
CMD ["node", "dist/index.js"]
