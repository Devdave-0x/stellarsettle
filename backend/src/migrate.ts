import "dotenv/config";
import { DatabaseSync } from "node:sqlite";
import { mkdirSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const migrationsDir = resolve(dirname(fileURLToPath(import.meta.url)), "../migrations");
const databasePath = process.env.DATABASE_PATH || "./data/stellarsettle.sqlite";
if (databasePath !== ":memory:") mkdirSync(dirname(resolve(databasePath)), { recursive: true });

const db = new DatabaseSync(databasePath);
db.exec("CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)");

const available = readdirSync(migrationsDir)
  .filter((file) => file.endsWith(".up.sql"))
  .map((file) => file.slice(0, -".up.sql".length))
  .sort();
const applied = new Set(
  (db.prepare("SELECT name FROM schema_migrations").all() as Array<{ name: string }>).map((row) => row.name),
);

const run = (name: string, direction: "up" | "down") => {
  const sql = readFileSync(join(migrationsDir, `${name}.${direction}.sql`), "utf8");
  db.exec("BEGIN");
  try {
    db.exec(sql);
    if (direction === "up") db.prepare("INSERT INTO schema_migrations (name) VALUES (?)").run(name);
    else db.prepare("DELETE FROM schema_migrations WHERE name = ?").run(name);
    db.exec("COMMIT");
  } catch (error) {
    db.exec("ROLLBACK");
    throw error;
  }
  console.log(`${direction === "up" ? "applied" : "rolled back"} ${name}`);
};

if (process.argv[2] === "down") {
  const last = available.filter((name) => applied.has(name)).at(-1);
  if (last) run(last, "down");
  else console.log("no migrations to roll back");
} else {
  const pending = available.filter((name) => !applied.has(name));
  pending.forEach((name) => run(name, "up"));
  if (pending.length === 0) console.log("no pending migrations");
}
db.close();
