import { formatParameter } from "@/lib/parameter-format"
import type { Parameter } from "@/lib/trial"

type ParameterItemProps = {
  /** 表示するパラメーター */
  parameter: Parameter
}

/**
 * パラメーター1件を「ラベル: 値」の1行で表示する。
 *
 * 1画面に多数のパラメーターが並ぶため、種別によらず高さを揃えて
 * 縦方向のスクロール量を抑える。
 */
const ParameterItem = ({ parameter }: ParameterItemProps) => {
  const { label, value } = formatParameter(parameter)

  return (
    <li data-slot="parameter-item" className="flex gap-2 py-1 text-sm">
      {label !== null && (
        <span className="shrink-0 text-muted-foreground">{label}</span>
      )}
      <span className="min-w-0 break-words">{value}</span>
    </li>
  )
}

export { ParameterItem }
export type { ParameterItemProps }
