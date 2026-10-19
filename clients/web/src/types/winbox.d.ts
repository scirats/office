declare module "winbox" {
  export interface WinBoxOptions {
    title?: string
    root?: HTMLElement
    mount?: HTMLElement
    x?: number | string
    y?: number | string
    width?: number | string
    height?: number | string
    minwidth?: number
    minheight?: number
    top?: number
    left?: number
    right?: number
    bottom?: number
    class?: string
    onfocus?: () => void
    onblur?: () => void
    onminimize?: () => void
    onrestore?: () => void
    onclose?: () => boolean | void
  }

  export default class WinBox {
    constructor(options?: WinBoxOptions)
    top: number | string
    left: number | string
    right: number | string
    bottom: number | string
    min: boolean
    max: boolean
    hidden: boolean
    header: number
    x: number
    y: number
    width: number
    height: number
    focus(): void
    show(animate?: boolean): void
    restore(): void
    maximize(state?: boolean): void
    resize(width?: number, height?: number, skipUpdate?: boolean): void
    move(x?: number, y?: number, skipUpdate?: boolean): void
    close(force?: boolean): void
  }
}
