export function blockFor(ms: number): void {
  const end = performance.now() + ms;
  while (performance.now() < end) {
    Math.sqrt(Math.random());
  }
}

export function startLoad(periodMs: number, maxBlockMs: number): () => void {
  const timer = setInterval(() => blockFor(Math.random() * maxBlockMs), periodMs);
  return () => clearInterval(timer);
}
