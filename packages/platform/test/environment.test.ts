import { expect, test, vi } from "vitest";
import { Environment, type EnvironmentData } from "../src/environment.ts";

test("environment reads once per snapshot and supplies defaults to missing fields", () => {
  const read = vi.fn<() => EnvironmentData>().mockReturnValueOnce({ screen: { width: 320 } })
    .mockReturnValueOnce({ colorScheme: "dark", screen: { height: 400 }, safeArea: { bottom: 12 } });
  const environment = new Environment(read);
  expect(read).not.toHaveBeenCalled();
  expect(environment.colorScheme).toBe("light");
  expect(environment.screen).toEqual({ width: 320, height: 0, scale: 1 });
  expect(environment.safeArea).toEqual({ top: 0, bottom: 0, start: 0, end: 0 });
  expect(environment.orientation).toBe("landscape");
  expect(read).toHaveBeenCalledTimes(1);
  environment.notify();
  expect(environment.colorScheme).toBe("dark");
  expect(environment.screen).toEqual({ width: 0, height: 400, scale: 1 });
  expect(environment.safeArea).toEqual({ top: 0, bottom: 12, start: 0, end: 0 });
  expect(environment.orientation).toBe("portrait");
  expect(read).toHaveBeenCalledTimes(2);
});

test("listeners see changed data in order; failures and removal are isolated", () => {
  let data: EnvironmentData = {};
  const environment = new Environment(() => data);
  const error = new Error("listener failed");
  const report = vi.spyOn(console, "error").mockImplementation(() => {});
  const seen: Array<string | number> = [];
  const remove = environment.onChange(() => { seen.push(environment.colorScheme); throw error; });
  const removeSecond = environment.onChange(() => { seen.push(environment.screen.height); });
  try {
    environment.notify();
    expect(seen).toEqual([]);
    data = { colorScheme: "dark", screen: { height: 30 } };
    environment.notify();
    expect(seen).toEqual(["dark", 30]);
    expect(report).toHaveBeenCalledExactlyOnceWith(error);
    remove();
    data = { screen: { height: 60 } };
    environment.notify();
    expect(seen).toEqual(["dark", 30, 60]);
  } finally {
    removeSecond();
    report.mockRestore();
  }
});
