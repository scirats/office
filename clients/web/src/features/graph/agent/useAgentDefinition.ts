import { useCallback, useEffect, useRef, useState } from "react"

import { useOfficeStore } from "@/lib/office/store"

import type { AgentDefinition } from "@/lib/office/protocol"

const AUTOSAVE_MS = 600

export type DefinitionPatch = Partial<Pick<AgentDefinition, "role" | "model" | "prompt" | "tools">>

export function useAgentDefinition(initial: AgentDefinition) {
  const defineAgent = useOfficeStore((state) => state.defineAgent)
  const [definition, setDefinition] = useState<AgentDefinition>(initial)
  const dirty = useRef(false)

  const update = useCallback((values: DefinitionPatch) => {
    dirty.current = true
    setDefinition((current) => ({ ...current, ...values }))
  }, [])

  useEffect(() => {
    if (!dirty.current) {
      return
    }
    if (definition.role.trim() === "" || definition.model === "") {
      return
    }
    const timer = setTimeout(() => {
      dirty.current = false
      defineAgent({
        name: definition.name,
        role: definition.role,
        prompt: definition.prompt,
        tools: definition.tools,
        model: definition.model,
      })
    }, AUTOSAVE_MS)
    return () => clearTimeout(timer)
  }, [definition, defineAgent])

  return { definition, update }
}
