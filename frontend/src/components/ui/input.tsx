import type { ComponentProps } from "react"

import { cn } from "@/lib/utils"

/**
 * テキスト入力のスタイル。
 *
 * `input` 以外の入力要素（`textarea` / `select`）にも同じ見た目を適用できるよう、
 * コンポーネントとは別にクラス名としても公開する。
 */
const inputClassName =
  "rounded-md border border-input bg-background px-3 py-1.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-50"

const Input = ({ className, ...props }: ComponentProps<"input">) => {
  return <input data-slot="input" className={cn(inputClassName, className)} {...props} />
}

export { Input, inputClassName }
