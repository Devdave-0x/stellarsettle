import assert from "node:assert/strict";
import { setTimeout as delay } from "node:timers/promises";
import { test } from "node:test";
import rateLimit from "@fastify/rate-limit";
import Fastify from "fastify";

const MAX = 3;
const WINDOW_MS = 300;

async function buildApp() {
  const app = Fastify();
  await app.register(rateLimit, {
    max: MAX,
    timeWindow: WINDOW_MS,
    keyGenerator: (request) => String(request.headers["x-wallet-address"] ?? request.ip),
  });
  app.get("/v1/api/events", async () => []);
  await app.ready();
  return app;
}

const hit = (app: Awaited<ReturnType<typeof buildApp>>, wallet = "GWALLET1") =>
  app.inject({ method: "GET", url: "/v1/api/events", headers: { "x-wallet-address": wallet } });

test("requests up to the limit succeed, the next gets 429, and the window resets", async () => {
  const app = await buildApp();
  try {
    for (let i = 0; i < MAX; i++) {
      assert.equal((await hit(app)).statusCode, 200);
    }
    assert.equal((await hit(app)).statusCode, 429);
    assert.equal((await hit(app, "GWALLET2")).statusCode, 200);

    await delay(WINDOW_MS + 100);
    assert.equal((await hit(app)).statusCode, 200);
  } finally {
    await app.close();
  }
});
