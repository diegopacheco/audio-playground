import { describe, expect, test } from "bun:test";
import { scale, toColumns } from "./plot";

describe("scale", () => {
  test("maps domain endpoints to canvas edges", () => {
    expect(scale(0, 0, 10, 200)).toBe(0);
    expect(scale(10, 0, 10, 200)).toBe(200);
    expect(scale(5, 0, 10, 200)).toBe(100);
  });

  test("degenerate domain does not produce NaN", () => {
    expect(scale(3, 3, 3, 100)).toBe(0);
  });
});

describe("toColumns", () => {
  test("keeps the tallest bin per column so the alignment spike is never hidden", () => {
    const bins = [
      { offset_sec: -10, count: 1 },
      { offset_sec: 1.99, count: 2 },
      { offset_sec: 2, count: 40 },
      { offset_sec: 2.01, count: 3 },
      { offset_sec: 10, count: 1 },
    ];
    const columns = toColumns(bins, 4);
    expect(columns).toHaveLength(4);
    const tallest = columns.reduce((a, b) => (b.count > a.count ? b : a));
    expect(tallest).toEqual({ count: 40, offset: 2 });
    expect(columns[0].count).toBe(1);
    expect(columns[3].count).toBe(1);
  });

  test("empty histogram gives empty columns", () => {
    expect(toColumns([], 3).every((c) => c.count === 0)).toBe(true);
  });
});
