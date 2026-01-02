import type { Options as MotionOptions } from "solid-motionone";
import { motion } from "./design-tokens";

/** Spring animation config for natural bounce */
export const springConfig = { stiffness: 300, damping: 24 };

/** Standard easing for UI animations */
export const easeOut = [0.22, 1, 0.36, 1] as const;

/** Bounce easing for playful animations */
export const easeBounce = [0.34, 1.56, 0.64, 1] as const;

/** Fade in animation */
export const fadeIn: MotionOptions = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  transition: { duration: motion.duration.fast / 1000, easing: easeOut },
};

/** Fade in while sliding up 20px */
export const fadeInUp: MotionOptions = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: motion.duration.normal / 1000, easing: easeOut },
};

/** Slide in from right */
export const slideInRight: MotionOptions = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  transition: { duration: motion.duration.normal / 1000, easing: easeOut },
};

/** Slide in from left */
export const slideInLeft: MotionOptions = {
  initial: { opacity: 0, x: -20 },
  animate: { opacity: 1, x: 0 },
  transition: { duration: motion.duration.slow / 1000, easing: easeOut },
};

/** Slide in from bottom */
export const slideInUp: MotionOptions = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: motion.duration.fast / 1000, easing: easeOut },
};

/** Scale in (pop) */
export const scaleIn: MotionOptions = {
  initial: { opacity: 0, scale: 0.95 },
  animate: { opacity: 1, scale: 1 },
  transition: { duration: motion.duration.fast / 1000, easing: easeOut },
};

/** Scale in with bounce effect */
export const scaleInBounce: MotionOptions = {
  initial: { opacity: 0, scale: 0.8 },
  animate: { opacity: 1, scale: 1 },
  transition: { duration: motion.duration.slow / 1000, easing: easeBounce },
};

/** Fade out animation */
export const fadeOut: MotionOptions = {
  initial: { opacity: 1 },
  animate: { opacity: 0 },
  transition: { duration: motion.duration.fast / 1000 },
};

/** Fade out while sliding down */
export const fadeOutDown: MotionOptions = {
  initial: { opacity: 1, y: 0 },
  animate: { opacity: 0, y: 20 },
  transition: { duration: motion.duration.fast / 1000 },
};

/** Scale out */
export const scaleOut: MotionOptions = {
  initial: { opacity: 1, scale: 1 },
  animate: { opacity: 0, scale: 0.95 },
  transition: { duration: motion.duration.fast / 1000 },
};

/** Quick scale out to zero */
export const scaleOutFast: MotionOptions = {
  initial: { opacity: 1, scale: 1 },
  animate: { opacity: 0, scale: 0 },
  transition: { duration: motion.duration.instant / 1000 },
};

/** Slide out left (for card dismissal) */
export const slideOutLeft: MotionOptions = {
  initial: { opacity: 1, x: 0 },
  animate: { opacity: 0, x: -100 },
  transition: { duration: motion.duration.normal / 1000, easing: easeOut },
};

/** Slide out right */
export const slideOutRight: MotionOptions = {
  initial: { opacity: 1, x: 0 },
  animate: { opacity: 0, x: 100 },
  transition: { duration: motion.duration.normal / 1000, easing: easeOut },
};

/** Card flip animation (3D) */
export const cardFlip: MotionOptions = {
  initial: { rotateY: 0 },
  animate: { rotateY: 180 },
  transition: { duration: 0.4, easing: easeOut },
};

/** Bounce in (for success feedback) */
export const bounceIn: MotionOptions = {
  initial: { opacity: 0, scale: 0.8 },
  animate: { opacity: 1, scale: 1 },
  transition: { duration: motion.duration.slow / 1000, easing: easeBounce },
};

/** Button press feedback - scale down slightly */
export const pressDown: MotionOptions = {
  initial: { scale: 1 },
  animate: { scale: 0.97 },
  transition: { duration: motion.duration.instant / 1000 },
};

/** Modal backdrop fade animation */
export const modalBackdrop: MotionOptions = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: motion.duration.normal / 1000 },
};

/** Modal content scale + fade animation */
export const modalContent: MotionOptions = {
  initial: { opacity: 0, scale: 0.9 },
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0.9 },
  transition: { duration: motion.duration.slow / 1000, easing: easeOut },
};

/** Stagger delay for list items */
export const staggerDelay = (index: number, baseDelay = 0.05) => index * baseDelay;

/**
 * Create staggered animation options for a list of items
 */
export const createStaggeredList = (
  count: number,
  baseAnimation: MotionOptions = fadeInUp,
  staggerMs = 50,
): MotionOptions[] => {
  return Array.from(
    { length: count },
    (_, index) => ({
      ...baseAnimation,
      transition: { ...baseAnimation.transition, delay: (index * staggerMs) / 1000 },
    }),
  );
};

/** CSS class names for keyframe animations */
export const cssAnimations = {
  pulse: "animate-pulse",
  shimmer: "animate-shimmer",
  breathe: "animate-breathe",
} as const;
