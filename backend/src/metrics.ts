// Minimal Prometheus text-format metrics (no external dependency).

const counters = {
  shipments_total: 0,
  milestones_confirmed_total: 0,
  rpc_errors_total: 0,
};

const buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10];
const bucketCounts = buckets.map(() => 0);
let durationSum = 0;
let durationCount = 0;

export function inc(name: keyof typeof counters): void {
  counters[name] += 1;
}

export function observeRequestDuration(seconds: number): void {
  buckets.forEach((le, i) => {
    if (seconds <= le) bucketCounts[i] = (bucketCounts[i] ?? 0) + 1;
  });
  durationSum += seconds;
  durationCount += 1;
}

export function renderMetrics(): string {
  const lines: string[] = [];
  for (const [name, value] of Object.entries(counters)) {
    lines.push(`# TYPE ${name} counter`, `${name} ${value}`);
  }
  lines.push("# TYPE request_duration_seconds histogram");
  buckets.forEach((le, i) => {
    lines.push(`request_duration_seconds_bucket{le="${le}"} ${bucketCounts[i] ?? 0}`);
  });
  lines.push(
    `request_duration_seconds_bucket{le="+Inf"} ${durationCount}`,
    `request_duration_seconds_sum ${durationSum}`,
    `request_duration_seconds_count ${durationCount}`,
  );
  return lines.join("\n") + "\n";
}
