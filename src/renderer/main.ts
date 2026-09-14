import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './assets/styles/index.css'
import './assets/styles/RunPage.css'

const savedTheme = localStorage.getItem('v1per-theme')
if (savedTheme) {
  document.documentElement.setAttribute('data-theme', savedTheme)
}

createApp(App).use(createPinia()).use(router).mount('#root')