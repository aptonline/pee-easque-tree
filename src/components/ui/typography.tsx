import * as React from "react"
import { cn } from "@/lib/utils"

export interface TypographyProps extends React.HTMLAttributes<HTMLParagraphElement> {}

export function TypographyMuted({ className, ...props }: TypographyProps) {
  return (
    <p
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  )
}
