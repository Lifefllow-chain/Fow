"use client";

import React, { forwardRef } from "react";
import { cn } from "@/lib/utils/cn";

/** Minimal label primitive used by the admin reporting filter panel. */
export const Label = forwardRef<
  HTMLLabelElement,
  React.LabelHTMLAttributes<HTMLLabelElement>
>(({ className, ...props }, ref) => (
  <label
    ref={ref}
    className={cn(
      "text-sm font-medium leading-none text-text-primary",
      className
    )}
    {...props}
  />
));
Label.displayName = "Label";

export default Label;
