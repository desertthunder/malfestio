import { describe, expect, it } from "vitest";
import { applyDensity, density, elevation, getElevation, motion, spacing, typography } from "../design-tokens";

describe("design-tokens", () => {
  describe("spacing", () => {
    it("provides 16px grid-based spacing values", () => {
      expect(spacing.xs).toBe(4);
      expect(spacing.sm).toBe(8);
      expect(spacing.md).toBe(12);
      expect(spacing.base).toBe(16);
      expect(spacing.lg).toBe(24);
      expect(spacing.xl).toBe(32);
      expect(spacing["2xl"]).toBe(48);
      expect(spacing["3xl"]).toBe(64);
      expect(spacing["4xl"]).toBe(96);
    });
  });

  describe("elevation", () => {
    it("provides 5 elevation levels", () => {
      expect(elevation["00"]).toEqual({ bg: "#161616", shadow: "none" });
      expect(elevation["01"].bg).toBe("#1E1E1E");
      expect(elevation["02"].bg).toBe("#262626");
      expect(elevation["03"].bg).toBe("#2C2C2C");
      expect(elevation["04"].bg).toBe("#323232");
    });

    it("increases shadow intensity with elevation", () => {
      expect(elevation["00"].shadow).toBe("none");
      expect(elevation["01"].shadow).toContain("0 1px 2px");
      expect(elevation["04"].shadow).toContain("0 8px 16px");
    });
  });

  describe("density", () => {
    it("provides three density multipliers", () => {
      expect(density.compact).toBe(0.75);
      expect(density.comfortable).toBe(1.0);
      expect(density.spacious).toBe(1.25);
    });
  });

  describe("motion", () => {
    it("provides duration values", () => {
      expect(motion.duration.instant).toBe(100);
      expect(motion.duration.fast).toBe(150);
      expect(motion.duration.normal).toBe(250);
      expect(motion.duration.slow).toBe(350);
      expect(motion.duration.slower).toBe(500);
    });

    it("provides easing curves", () => {
      expect(motion.easing.standard).toBe("cubic-bezier(0.4, 0.0, 0.2, 1)");
      expect(motion.easing.accelerate).toBe("cubic-bezier(0.4, 0.0, 1, 1)");
      expect(motion.easing.decelerate).toBe("cubic-bezier(0.0, 0.0, 0.2, 1)");
      expect(motion.easing.sharp).toBe("cubic-bezier(0.4, 0.0, 0.6, 1)");
    });
  });

  describe("typography", () => {
    it("provides font scale with line heights", () => {
      expect(typography.scale.xs).toEqual({ size: 12, lineHeight: 16 });
      expect(typography.scale.base).toEqual({ size: 16, lineHeight: 24 });
      expect(typography.scale["4xl"]).toEqual({ size: 36, lineHeight: 40 });
    });
  });

  describe("applyDensity", () => {
    it("applies compact density multiplier", () => {
      expect(applyDensity(16, "compact")).toBe(12);
      expect(applyDensity(24, "compact")).toBe(18);
    });

    it("applies comfortable density multiplier (default)", () => {
      expect(applyDensity(16)).toBe(16);
      expect(applyDensity(24, "comfortable")).toBe(24);
    });

    it("applies spacious density multiplier", () => {
      expect(applyDensity(16, "spacious")).toBe(20);
      expect(applyDensity(24, "spacious")).toBe(30);
    });

    it("rounds results to nearest integer", () => {
      expect(applyDensity(17, "compact")).toBe(13);
      expect(applyDensity(17, "spacious")).toBe(21);
    });
  });

  describe("getElevation", () => {
    it("returns elevation object for given level", () => {
      const level01 = getElevation("01");
      expect(level01).toEqual({ bg: "#1E1E1E", shadow: "0 1px 2px rgba(0, 0, 0, 0.1)" });
    });

    it("returns background layer for level 00", () => {
      const level00 = getElevation("00");
      expect(level00.shadow).toBe("none");
    });
  });
});
