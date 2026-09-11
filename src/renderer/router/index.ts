import { createRouter, createWebHashHistory } from 'vue-router'
import DriversView from '@/views/DriversView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/mediatek' },
    { path: '/mediatek', name: 'MediaTek', component: () => import('@/pages/Mediatek/index.vue') },
    {
      path: '/mediatek/flash',
      name: 'MediaTekFlash',
      component: () => import('@/pages/MediatekFlash/index.vue'),
    },
    { path: '/drivers', name: 'Drivers', component: DriversView },
    {
      path: '/drivers/run',
      name: 'DriverRun',
      component: () => import('@/views/DriverRun.vue'),
    },
    {
      path: '/unisoc',
      name: 'Unisoc',
      component: () => import('@/pages/Unisoc/index.vue'),
    },
    {
      path: '/unisoc/run',
      name: 'UnisocRun',
      component: () => import('@/pages/Unisoc/run.vue'),
    },
    {
      path: '/utilities',
      name: 'Utilities',
      component: () => import('@/pages/Utilities/index.vue'),
    },
    {
      path: '/utilities/run',
      name: 'UtilitiesRun',
      component: () => import('@/pages/Utilities/run.vue'),
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