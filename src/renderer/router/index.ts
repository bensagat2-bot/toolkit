import { createRouter, createWebHashHistory } from 'vue-router'
import DriversView from '@/views/DriversView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/drivers' },
    { path: '/drivers', name: 'Drivers', component: DriversView },
    {
      path: '/mediatek',
      name: 'MediaTek',
      component: () => import('@/pages/Mediatek/index.vue'),
    },
    {
      path: '/unisoc',
      name: 'Unisoc',
      component: () => import('@/pages/Unisoc/index.vue'),
    },
    {
      path: '/utilities',
      name: 'Utilities',
      component: () => import('@/pages/Utilities/index.vue'),
    },
    {
      path: '/settings',
      name: 'Settings',
      component: () => import('@/pages/Settings/index.vue'),
    },
    { path: '/:pathMatch(.*)*', redirect: '/drivers' },
  ],
})

export default router