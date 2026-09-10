import Event from './Event'

declare class keyEventTypes extends Event {
  on(event: string, listener: (event: LX.KeyDownEevent) => any): void
  off(event: string, listener: (event: LX.KeyDownEevent) => any): void
}

export type KeyEventTypes = keyEventTypes

export const createKeyEventHub = (): keyEventTypes => {
  return new Event()
}

export const registerKeyEvent = () => {
}

export const unregisterKeyEvent = () => {
}

export const clearDownKeys = () => {
}