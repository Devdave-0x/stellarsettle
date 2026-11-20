// Minimal Prometheus exposition-format metrics (issue #84).

const BUCKETS = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10];

type Histogram = { counts: number[]; sum: number; count: number };

const requests = new Map<string, number>();
const latencies = new Map<string, Histogram>();
const polls = { success: 0, failure: 0 };

const escape = (value: string) => value.replace(/\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\n");

export function recordRequest(method: string, route: string, status: number, durationMs: number): void {
  const key = `method="${escape(method)}",route="${escape(route)}"`;
  const countKey = `${key},status="${status}"`;
  requests.set(countKey, (requests.get(countKey) ?? 0) + 1);
  const seconds = durationMs / 1000;
  const hist = latencies.get(key) ?? { counts: BUCKETS.map(() => 0), sum: 0, count: 0 };
  BUCKETS.forEach((bound, i) => {
    if (seconds <= bound) hist.counts[i] = (hist.counts[i] ?? 0) + 1;
  });
  hist.sum += seconds;
  hist.count += 1;
  latencies.set(key, hist);
}

export function recordPoll(success: boolean): void {
  if (success) polls.success += 1;
  else polls.failure += 1;
}

export function renderMetrics(): string {
  const lines: string[] = [
    "# HELP http_requests_total Total HTTP requests.",
    "# TYPE http_requests_total counter",
  ];
  for (const [labels, value] of requests) lines.push(`http_requests_total{${labels}} ${value}`);
  lines.push(
    "# HELP http_request_duration_seconds HTTP request latency in seconds.",
    "# TYPE http_request_duration_seconds histogram",
  );
  for (const [labels, hist] of latencies) {
    BUCKETS.forEach((bound, i) => {
      lines.push(`http_request_duration_seconds_bucket{${labels},le="${bound}"} ${hist.counts[i] ?? 0}`);
    });
    lines.push(`http_request_duration_seconds_bucket{${labels},le="+Inf"} ${hist.count}`);
    lines.push(`http_request_duration_seconds_sum{${labels}} ${hist.sum}`);
    lines.push(`http_request_duration_seconds_count{${labels}} ${hist.count}`);
  }
  lines.push(
    "# HELP event_poll_total Contract event poll attempts by result.",
    "# TYPE event_poll_total counter",
    `event_poll_total{result="success"} ${polls.success}`,
    `event_poll_total{result="failure"} ${polls.failure}`,
  );
  return `${lines.join("\n")}\n`;
}
