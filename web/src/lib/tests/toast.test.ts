import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { toast, toasts } from "../toast";

describe("Toast Store", () => {
  beforeEach(() => {
    const currentToasts = toasts();
    currentToasts.forEach(t => toast.remove(t.id));
  });

  it("should add a toast", () => {
    createRoot(() => {
      toast.success("Success message");
      const currentToasts = toasts();
      expect(currentToasts.length).toBe(1);
      expect(currentToasts[0].message).toBe("Success message");
      expect(currentToasts[0].type).toBe("success");
    });
  });

  it("should remove a toast", () => {
    createRoot(() => {
      toast.error("Error message");
      let currentToasts = toasts();
      expect(currentToasts.length).toBe(1);
      const id = currentToasts[0].id;

      toast.remove(id);
      currentToasts = toasts();
      expect(currentToasts.length).toBe(0);
    });
  });

  it("should auto-dismiss toast", () => {
    vi.useFakeTimers();
    createRoot(() => {
      toast.info("Info message", 1000);
      expect(toasts().length).toBe(1);

      vi.advanceTimersByTime(1000);
      expect(toasts().length).toBe(0);
    });
    vi.useRealTimers();
  });
});
