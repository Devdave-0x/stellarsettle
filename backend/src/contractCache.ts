type Entry = { value: unknown; expiresAt: number };

export class ContractCache {
  private readonly entries = new Map<string, Entry>();

  constructor(
    private readonly ttlMs: number,
    private readonly log?: { debug: (obj: object, msg: string) => void },
    private readonly now: () => number = Date.now,
  ) {}

  async get(key: string, load: () => Promise<unknown>): Promise<unknown> {
    const entry = this.entries.get(key);
    if (entry && entry.expiresAt > this.now()) {
      this.log?.debug({ key }, "contract cache hit");
      return entry.value;
    }
    this.entries.delete(key);
    this.log?.debug({ key }, "contract cache miss");
    const value = await load();
    this.entries.set(key, { value, expiresAt: this.now() + this.ttlMs });
    return value;
  }
}
