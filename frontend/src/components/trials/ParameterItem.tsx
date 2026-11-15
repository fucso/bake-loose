import type { ReactNode } from "react"

import { formatParameter } from "@/lib/parameter-format"
import type { Parameter } from "@/lib/trial"

type ParameterItemProps = {
  /** 表示するパラメーター */
  parameter: Parameter
  /**
   * 行末に並べる操作 UI（編集・削除など）。
   * 記録操作を行えない場合に行の高さが変わらないよう、省略時は何も描画しない。
   */
  actions?: ReactNode
}

/**
 * パラメーター1件を「ラベル: 値」の1行で表示する。
 *
 * 1画面に多数のパラメーターが並ぶため、種別によらず高さを揃えて
 * 縦方向のスクロール量を抑える。
 */
const ParameterItem = ({ parameter, actions }: ParameterItemProps) => {
  const { label, value } = formatParameter(parameter)

  return (
    <li data-slot="parameter-item" className="flex items-center gap-2 py-1 text-sm">
      {label !== null && (
        <span className="shrink-0 text-muted-foreground">{label}</span>
      )}
      <span className="min-w-0 flex-1 break-words">{value}</span>
      {actions}
    </li>
  )
}

export { ParameterItem }
export type { ParameterItemProps }
