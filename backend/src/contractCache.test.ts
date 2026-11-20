import assert from "node:assert/strict";
import { test } from "node:test";
import { ContractCache } from "./contractCache.ts";

test("contract cache serves hits within the TTL and evicts stale entries", async () => {
  let now = 0;
  let loads = 0;
  const cache = new ContractCache(5_000, undefined, () => now);
  const load = async () => ++loads;

  assert.equal(await cache.get("shipment:1", load), 1);
  now = 4_999;
  assert.equal(await cache.get("shipment:1", load), 1);
  now = 5_000;
  assert.equal(await cache.get("shipment:1", load), 2);
  assert.equal(loads, 2);
});
