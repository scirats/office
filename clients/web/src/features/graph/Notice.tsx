import { useEffect } from "react"

const DISMISS_MS = 5000

export function Notice({ message, onDismiss }: { message: string | null; onDismiss(): void }) {
  useEffect(() => {
    if (message === null) {
      return
    }
    const timer = setTimeout(onDismiss, DISMISS_MS)
    return () => clearTimeout(timer)
  }, [message, onDismiss])

  if (message === null) {
    return null
  }

  return (
    <div className="pointer-events-none fixed bottom-4 left-1/2 z-50 -translate-x-1/2 rounded-lg border border-destructive/40 bg-card px-3 py-1.5 text-xs text-destructive shadow-md">
      {message}
    </div>
  )
}
