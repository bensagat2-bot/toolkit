import { createRouter, createWebHashHistory } from 'vue-router'
import DriversView from '@/views/DriversView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/mediatek' },
    { path: '/mediatek', name: 'MediaTek', component: () => import('@/pages/Mediatek/index.vue') },
    { path: '/drivers', name: 'Drivers', component: DriversView },
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
    { path: '/:pathMatch(.*)*', redirect: '/mediatek' },
  ],
})

export default router