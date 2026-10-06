export type OfficeEvent = {
  id: string
  type: string
  subject?: string
  correlationid?: string
  data: Record<string, unknown>
}

type EventListener = (event: OfficeEvent) => void

const SUBPROTOCOL = "office.v1"
const SOURCE = "/office/client/web"
const RECONNECT_DELAY_MS = 1000

export class OfficeClient {
  private socket: WebSocket | null = null
  private readonly url: string
  private readonly listeners = new Set<EventListener>()
  private readonly pending: string[] = []
  private reconnectTimer: number | null = null
  private closedByUser = false

  constructor(url: string) {
    this.url = url
  }

  connect(): void {
    if (this.socket !== null) {
      return
    }
    this.closedByUser = false
    this.open()
  }

  onEvent(listener: EventListener): () => void {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  createWorkspace(name: string): void {
    this.send("office.workspace.create.v1", { name })
  }

  defineAgent(definition: {
    name: string
    role: string
    prompt: string
    tools: string[]
  }): void {
    this.send("office.agent.definition.create.v1", definition)
  }

  close(): void {
    this.closedByUser = true
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    const socket = this.socket
    this.socket = null
    if (socket === null) {
      return
    }
    if (socket.readyState === WebSocket.CONNECTING) {
      socket.onopen = () => socket.close()
    } else {
      socket.close()
    }
  }

  private open(): void {
    const socket = new WebSocket(this.url, SUBPROTOCOL)
    this.socket = socket

    socket.onopen = () => {
      if (this.socket !== socket) {
        return
      }
      this.send("office.subscription.add.v1", { filter: {} })
      this.send("office.workspace.list.v1", {})
      for (const payload of this.pending.splice(0)) {
        socket.send(payload)
      }
    }
    socket.onmessage = (message) => {
      if (this.socket !== socket) {
        return
      }
      let event: OfficeEvent
      try {
        event = JSON.parse(String(message.data)) as OfficeEvent
      } catch {
        return
      }
      for (const listener of this.listeners) {
        listener(event)
      }
    }
    socket.onclose = () => {
      if (this.socket !== socket) {
        return
      }
      this.socket = null
      if (!this.closedByUser) {
        this.scheduleReconnect()
      }
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer !== null) {
      return
    }
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null
      if (this.closedByUser) {
        return
      }
      if (this.socket === null) {
        this.open()
      } else {
        this.scheduleReconnect()
      }
    }, RECONNECT_DELAY_MS)
  }

  private send(type: string, data: Record<string, unknown>): void {
    const payload = JSON.stringify({
      specversion: "1.0",
      id: crypto.randomUUID(),
      source: SOURCE,
      type,
      data,
    })
    if (this.socket !== null && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(payload)
    } else {
      this.pending.push(payload)
    }
  }
}
