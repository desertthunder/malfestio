import { describe, expect, it } from "vitest";
import {
  bounceIn,
  cardFlip,
  createStaggeredList,
  cssAnimations,
  easeBounce,
  easeOut,
  fadeIn,
  fadeInUp,
  fadeOut,
  fadeOutDown,
  modalBackdrop,
  modalContent,
  pressDown,
  scaleIn,
  scaleInBounce,
  scaleOut,
  scaleOutFast,
  slideInLeft,
  slideInRight,
  slideInUp,
  slideOutLeft,
  slideOutRight,
  springConfig,
  staggerDelay,
} from "../animations";

describe("animations", () => {
  describe("config", () => {
    it("exports spring config with stiffness and damping", () => {
      expect(springConfig.stiffness).toBe(300);
      expect(springConfig.damping).toBe(24);
    });

    it("exports easing curves as tuples", () => {
      expect(easeOut).toHaveLength(4);
      expect(easeBounce).toHaveLength(4);
    });
  });

  describe("entrance animations", () => {
    it.each([
      ["fadeIn", fadeIn],
      ["fadeInUp", fadeInUp],
      ["slideInRight", slideInRight],
      ["slideInLeft", slideInLeft],
      ["slideInUp", slideInUp],
      ["scaleIn", scaleIn],
      ["scaleInBounce", scaleInBounce],
    ])("%s has required properties", (_, preset) => {
      expect(preset).toHaveProperty("initial");
      expect(preset).toHaveProperty("animate");
      expect(preset).toHaveProperty("transition");
    });

    it("fadeInUp starts from y: 20", () => {
      expect(fadeInUp.initial).toEqual({ opacity: 0, y: 20 });
      expect(fadeInUp.animate).toEqual({ opacity: 1, y: 0 });
    });

    it("slideInLeft starts from x: -20", () => {
      expect(slideInLeft.initial).toEqual({ opacity: 0, x: -20 });
    });
  });

  describe("exit animations", () => {
    it.each([
      ["fadeOut", fadeOut],
      ["fadeOutDown", fadeOutDown],
      ["scaleOut", scaleOut],
      ["scaleOutFast", scaleOutFast],
      ["slideOutLeft", slideOutLeft],
      ["slideOutRight", slideOutRight],
    ])("%s has required properties", (_, preset) => {
      expect(preset).toHaveProperty("initial");
      expect(preset).toHaveProperty("animate");
      expect(preset).toHaveProperty("transition");
    });

    it("fadeOutDown animates to y: 20", () => {
      expect(fadeOutDown.animate).toEqual({ opacity: 0, y: 20 });
    });

    it("slideOutRight animates to x: 100", () => {
      expect(slideOutRight.animate).toEqual({ opacity: 0, x: 100 });
    });
  });

  describe("interactive animations", () => {
    it("cardFlip rotates 180 degrees", () => {
      expect(cardFlip.animate).toEqual({ rotateY: 180 });
    });

    it("bounceIn uses bounce easing", () => {
      expect(bounceIn.transition?.easing).toEqual(easeBounce);
    });

    it("pressDown scales to 0.97", () => {
      expect(pressDown.animate).toEqual({ scale: 0.97 });
    });
  });

  describe("modal animations", () => {
    it("modalBackdrop has exit property", () => {
      expect(modalBackdrop).toHaveProperty("exit");
      expect(modalBackdrop.exit).toEqual({ opacity: 0 });
    });

    it("modalContent has scale and fade", () => {
      expect(modalContent.initial).toEqual({ opacity: 0, scale: 0.9 });
      expect(modalContent.animate).toEqual({ opacity: 1, scale: 1 });
      expect(modalContent.exit).toEqual({ opacity: 0, scale: 0.9 });
    });
  });

  describe("staggerDelay", () => {
    it("calculates delay based on index", () => {
      expect(staggerDelay(0)).toBe(0);
      expect(staggerDelay(1)).toBe(0.05);
      expect(staggerDelay(5)).toBe(0.25);
    });

    it("accepts custom base delay", () => {
      expect(staggerDelay(2, 0.1)).toBe(0.2);
      expect(staggerDelay(3, 0.02)).toBe(0.06);
    });
  });

  describe("createStaggeredList", () => {
    it("creates array of motion options with staggered delays", () => {
      const list = createStaggeredList(3);

      expect(list).toHaveLength(3);
      expect(list[0].transition?.delay).toBe(0);
      expect(list[1].transition?.delay).toBe(0.05);
      expect(list[2].transition?.delay).toBe(0.1);
    });

    it("uses provided base animation", () => {
      const list = createStaggeredList(2, fadeIn);

      expect(list[0].initial).toEqual(fadeIn.initial);
      expect(list[0].animate).toEqual(fadeIn.animate);
    });

    it("accepts custom stagger duration", () => {
      const list = createStaggeredList(3, fadeInUp, 100);

      expect(list[0].transition?.delay).toBe(0);
      expect(list[1].transition?.delay).toBe(0.1);
      expect(list[2].transition?.delay).toBe(0.2);
    });
  });

  describe("cssAnimations", () => {
    it("exports CSS class names for keyframe animations", () => {
      expect(cssAnimations.pulse).toBe("animate-pulse");
      expect(cssAnimations.shimmer).toBe("animate-shimmer");
      expect(cssAnimations.breathe).toBe("animate-breathe");
    });
  });
});
