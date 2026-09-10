import Event from './Event'


export class AppEvent extends Event {
  configUpdate(setting: Partial<LX.AppSetting>) {
    this.emit('configUpdate', setting)
  }

  focus() {
    this.emit('focus')
  }

  dragStart() {
    this.emit('dragStart')
  }

  dragEnd() {
    this.emit('dragEnd')
  }

  keyDown(event: LX.KeyDownEevent) {
    this.emit('keyDown', event)
  }
}


type EventMethods = Omit<EventType, keyof Event>


declare class EventType extends AppEvent {
  on<K extends keyof EventMethods>(event: K, listener: EventMethods[K]): any
  off<K extends keyof EventMethods>(event: K, listener: EventMethods[K]): any
}

export type AppEventTypes = Omit<EventType, keyof Omit<Event, 'on' | 'off'>>
export const createAppEventHub = (): AppEventTypes => {
  return new AppEvent()
}
