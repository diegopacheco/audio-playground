import { expect, test } from "bun:test";
import { streamUrl } from "./api";

test("stream url follows the page scheme so the Vite proxy and TLS both work", () => {
  const params = { sample_rate: 48000, window: "hann" as const, bars: 64 };
  expect(streamUrl(params, { protocol: "http:", host: "h:1" })).toBe(
    "ws://h:1/api/stream?sample_rate=48000&window=hann&bars=64",
  );
  expect(streamUrl(params, { protocol: "https:", host: "h" })).toStartWith("wss://h/api/stream?");
});
