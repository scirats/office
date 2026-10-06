declare interface WinBoxControl {
  class?: string
  image?: string
  click?: (this: HTMLElement, event: MouseEvent, winbox: WinBox) => void
  index?: number
}

declare interface WinBoxParams {
  id?: string
  index?: number
  root?: Node
  title?: string
  icon?: string
  mount?: Node
  html?: string
  url?: string
  x?: "right" | "center" | string | number
  y?: "bottom" | "center" | string | number
  width?: string | number
  height?: string | number
  minwidth?: string | number
  minheight?: string | number
  maxwidth?: string | number
  maxheight?: string | number
  left?: string | number
  right?: string | number
  top?: string | number
  bottom?: string | number
  min?: boolean
  max?: boolean
  hidden?: boolean
  modal?: boolean
  overflow?: boolean
  background?: string
  border?: string | number
  header?: number
  class?: string | string[]
  oncreate?: (this: WinBox, params?: WinBoxParams) => void
  onclose?: (this: WinBox, force?: boolean) => boolean | undefined
  onfocus?: (this: WinBox) => void
  onblur?: (this: WinBox) => void
  onmove?: (this: WinBox, x: number, y: number) => void
  onresize?: (this: WinBox, width: number, height: number) => void
  onfullscreen?: (this: WinBox) => void
  onminimize?: (this: WinBox) => void
  onmaximize?: (this: WinBox) => void
  onrestore?: (this: WinBox) => void
  onhide?: (this: WinBox) => void
  onshow?: (this: WinBox) => void
  onload?: (this: WinBox) => void
}

declare interface WinBox {
  id: string | number
  title: string
  dom: HTMLElement
  window: Node
  body: HTMLElement
  index: number
  x: string | number
  y: string | number
  width: string | number
  height: string | number
  minwidth: string | number
  minheight: string | number
  maxwidth: string | number
  maxheight: string | number
  left: string | number
  right: string | number
  top: string | number
  bottom: string | number
  overflow: boolean
  min: boolean
  max: boolean
  full: boolean
  hidden: boolean
  focused: boolean
  mount(src?: Element): WinBox
  unmount(dest?: Element): WinBox
  setTitle(title: string): WinBox
  setIcon(url: string): WinBox
  setBackground(background: string): WinBox
  focus(state?: boolean): WinBox
  blur(state?: boolean): WinBox
  hide(state?: boolean): WinBox
  show(state?: boolean): WinBox
  minimize(state?: boolean): WinBox
  restore(): WinBox
  maximize(state?: boolean): WinBox
  fullscreen(state?: boolean): WinBox
  close(force?: boolean): boolean | undefined
  move(x?: string | number, y?: string | number, skipUpdate?: boolean): WinBox
  resize(width?: string | number, height?: string | number, skipUpdate?: boolean): WinBox
  addControl(control: WinBoxControl): WinBox
  removeControl(control: string): WinBox
  addClass(classname: string): WinBox
  removeClass(classname: string): WinBox
  hasClass(classname: string): WinBox
  toggleClass(classname: string): WinBox
}

declare const WinBox: {
  (title: string, params?: WinBoxParams): WinBox
  (params: WinBoxParams): WinBox
  new (title: string, params?: WinBoxParams): WinBox
  new (params: WinBoxParams): WinBox
  stack(): WinBox[]
}

export default WinBox
