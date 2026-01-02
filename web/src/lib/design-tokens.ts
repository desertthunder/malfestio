export const spacing = { xs: 4, sm: 8, md: 12, base: 16, lg: 24, xl: 32, "2xl": 48, "3xl": 64, "4xl": 96 } as const;

export const elevation = {
  "00": { bg: "#161616", shadow: "none" },
  "01": { bg: "#1E1E1E", shadow: "0 1px 2px rgba(0, 0, 0, 0.1)" },
  "02": { bg: "#262626", shadow: "0 2px 4px rgba(0, 0, 0, 0.15)" },
  "03": { bg: "#2C2C2C", shadow: "0 4px 8px rgba(0, 0, 0, 0.2)" },
  "04": { bg: "#323232", shadow: "0 8px 16px rgba(0, 0, 0, 0.25)" },
} as const;

export const density = { compact: 0.75, comfortable: 1.0, spacious: 1.25 } as const;

export const motion = {
  duration: { instant: 100, fast: 150, normal: 250, slow: 350, slower: 500 },
  easing: {
    standard: "cubic-bezier(0.4, 0.0, 0.2, 1)",
    accelerate: "cubic-bezier(0.4, 0.0, 1, 1)",
    decelerate: "cubic-bezier(0.0, 0.0, 0.2, 1)",
    sharp: "cubic-bezier(0.4, 0.0, 0.6, 1)",
  },
} as const;

export const typography = {
  scale: {
    xs: { size: 12, lineHeight: 16 },
    sm: { size: 14, lineHeight: 20 },
    base: { size: 16, lineHeight: 24 },
    lg: { size: 18, lineHeight: 28 },
    xl: { size: 20, lineHeight: 28 },
    "2xl": { size: 24, lineHeight: 32 },
    "3xl": { size: 30, lineHeight: 36 },
    "4xl": { size: 36, lineHeight: 40 },
  },
} as const;

export type DensityMode = keyof typeof density;
export type SpacingKey = keyof typeof spacing;
export type ElevationLevel = keyof typeof elevation;

/**
 * Helper to calculate spacing with density multiplier
 */
export function applyDensity(baseSpacing: number, densityMode: DensityMode = "comfortable"): number {
  return Math.round(baseSpacing * density[densityMode]);
}

/**
 * Helper to get elevation styles
 */
export function getElevation(level: ElevationLevel) {
  return elevation[level];
}
