// declare module '*.vue' {
//   import { App } from 'vue'
//   export default App.Component
// }

declare module '*.vue' {
  import { type Component } from 'vue'
  const component: Component
  export default component
}

declare module '*.png' {
  const value: string
  export default value
}

declare module '*.jpg' {
  const value: string
  export default value
}

declare module '*.svg' {
  const value: string
  export default value
}
