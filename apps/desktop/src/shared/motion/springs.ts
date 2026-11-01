import type { Transition } from "motion/react";

export const springs = {
  island: { type: "spring", stiffness: 420, damping: 34, mass: 0.9 },
} satisfies Record<string, Transition>;

export const instant: Transition = { duration: 0 };
