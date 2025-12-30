import type { Options as MotionOptions } from "solid-motionone";

/** Spring animation config for natural bounce */
export const springConfig = { stiffness: 300, damping: 24 };

/** Standard easing for UI animations */
export const easeOut = [0.22, 1, 0.36, 1] as const;

/** Fade in animation */
export const fadeIn: MotionOptions = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  transition: { duration: 0.2, easing: easeOut },
};

/** Fade out animation */
export const fadeOut: MotionOptions = {
  initial: { opacity: 1 },
  animate: { opacity: 0 },
  transition: { duration: 0.15 },
};

/** Slide in from right */
export const slideInRight: MotionOptions = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  transition: { duration: 0.25, easing: easeOut },
};

/** Slide in from bottom */
export const slideInUp: MotionOptions = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.2, easing: easeOut },
};

/** Scale in (pop) */
export const scaleIn: MotionOptions = {
  initial: { opacity: 0, scale: 0.95 },
  animate: { opacity: 1, scale: 1 },
  transition: { duration: 0.2, easing: easeOut },
};

/** Scale out */
export const scaleOut: MotionOptions = {
  initial: { opacity: 1, scale: 1 },
  animate: { opacity: 0, scale: 0.95 },
  transition: { duration: 0.15 },
};

/** Stagger delay for list items */
export const staggerDelay = (index: number, baseDelay = 0.05) => index * baseDelay;
