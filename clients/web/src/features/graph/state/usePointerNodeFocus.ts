import { useEffect } from "react"
import { useReactFlow } from "@xyflow/react"

export function usePointerNodeFocus(onHover: (id: string | null) => void): void {
  const { screenToFlowPosition, getNodes } = useReactFlow()

  useEffect(() => {
    const onPointerMove = (event: PointerEvent) => {
      const point = screenToFlowPosition({ x: event.clientX, y: event.clientY })
      const hit = getNodes().find((node) => {
        const width = node.measured?.width ?? 0
        const height = node.measured?.height ?? 0
        return (
          point.x >= node.position.x &&
          point.x <= node.position.x + width &&
          point.y >= node.position.y &&
          point.y <= node.position.y + height
        )
      })
      onHover(hit?.id ?? null)
    }

    window.addEventListener("pointermove", onPointerMove, true)
    return () => window.removeEventListener("pointermove", onPointerMove, true)
  }, [screenToFlowPosition, getNodes, onHover])
}
