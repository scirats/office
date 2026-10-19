import { parseOfficeEvent, type OfficeEvent, type WireCommand } from "./protocol"

const SUBPROTOCOL = "office.v1"
const DEFAULT_SOURCE = "/office/client/web"
const RECONNECT_BASE_MS = 500
const RECONNECT_MAX_MS = 8000

export type ConnectionState = "connecting" | "open" | "closed"

export interface OfficeClientOptions {
  url: string
  source?: string
}

export class OfficeClient {
  private socket: WebSocket | null = null
  private readonly url: string
  private readonly source: string
  private readonly eventListeners = new Set<(event: OfficeEvent) => void>()
  private readonly openListeners = new Set<() => void>()
  private readonly stateListeners = new Set<(state: ConnectionState) => void>()
  private readonly outbox: string[] = []
  private reconnectAttempts = 0
  private reconnectTimer: number | null = null
  private closedByUser = false
  private state: ConnectionState = "closed"

  constructor(options: OfficeClientOptions) {
    this.url = options.url
    this.source = options.source ?? DEFAULT_SOURCE
  }

  connect(): void {
    if (this.socket !== null) {
      return
    }
    this.closedByUser = false
    this.open()
  }

  close(): void {
    this.closedByUser = true
    this.clearReconnectTimer()
    const socket = this.socket
    this.socket = null
    this.setState("closed")
    if (socket === null) {
      return
    }
    if (socket.readyState === WebSocket.CONNECTING) {
      socket.onopen = () => socket.close()
    } else {
      socket.close()
    }
  }

  onEvent(listener: (event: OfficeEvent) => void): () => void {
    this.eventListeners.add(listener)
    return () => {
      this.eventListeners.delete(listener)
    }
  }

  onOpen(listener: () => void): () => void {
    this.openListeners.add(listener)
    return () => {
      this.openListeners.delete(listener)
    }
  }

  onStateChange(listener: (state: ConnectionState) => void): () => void {
    this.stateListeners.add(listener)
    listener(this.state)
    return () => {
      this.stateListeners.delete(listener)
    }
  }

  send(command: WireCommand): string {
    const id = crypto.randomUUID()
    const payload = JSON.stringify({
      specversion: "1.0",
      id,
      source: this.source,
      type: command.type,
      data: command.data,
    })
    if (this.isOpen()) {
      this.socket?.send(payload)
    } else {
      this.outbox.push(payload)
    }
    return id
  }

  private isOpen(): boolean {
    return this.socket !== null && this.socket.readyState === WebSocket.OPEN
  }

  private open(): void {
    this.setState("connecting")
    const socket = new WebSocket(this.url, SUBPROTOCOL)
    this.socket = socket

    socket.onopen = () => {
      if (this.socket !== socket) {
        return
      }
      this.reconnectAttempts = 0
      this.setState("open")
      for (const payload of this.outbox.splice(0)) {
        socket.send(payload)
      }
      for (const listener of this.openListeners) {
        listener()
      }
    }

    socket.onmessage = (message) => {
      if (this.socket !== socket) {
        return
      }
      const result = parseOfficeEvent(String(message.data))
      if (result.kind === "invalid") {
        console.warn(`[office] dropped event: ${result.reason}`)
        return
      }
      if (result.kind === "ignored") {
        return
      }
      for (const listener of this.eventListeners) {
        listener(result.event)
      }
    }

    socket.onclose = () => {
      if (this.socket !== socket) {
        return
      }
      this.socket = null
      this.setState("closed")
      if (!this.closedByUser) {
        this.scheduleReconnect()
      }
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer !== null) {
      return
    }
    const delay = Math.min(RECONNECT_BASE_MS * 2 ** this.reconnectAttempts, RECONNECT_MAX_MS)
    this.reconnectAttempts += 1
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null
      if (!this.closedByUser && this.socket === null) {
        this.open()
      }
    }, delay)
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  private setState(state: ConnectionState): void {
    if (this.state === state) {
      return
    }
    this.state = state
    for (const listener of this.stateListeners) {
      listener(state)
    }
  }
}
