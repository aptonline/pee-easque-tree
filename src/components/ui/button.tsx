import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive:
          "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline:
          "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, style, ...props }, ref) => {
    const Comp = asChild ? Slot : "button"

    // Apply theme color for default and secondary variants
    const shouldUseThemeColor = variant === "default" || variant === "secondary" || !variant

    const themeStyle = shouldUseThemeColor ? {
      backgroundColor: 'var(--theme-color) !important' as any,
      borderColor: 'rgba(var(--theme-color-rgb), 0.5) !important' as any,
      boxShadow: '0 2px 10px rgba(var(--theme-color-rgb), 0.3) !important' as any,
      color: '#000000 !important' as any,
      ...style
    } : style

    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        style={themeStyle}
        data-theme-button={shouldUseThemeColor ? "true" : undefined}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }
