import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/unisoc',
    },
    {
      path: '/unisoc',
      name: 'Unisoc',
      component: () => import('./pages/Unisoc/index.vue'),
      meta: { name: 'Unisoc Tools' },
    },
    {
      path: '/mediatek',
      name: 'Mediatek',
      component: () => import('./pages/Mediatek/index.vue'),
      meta: { name: 'MediaTek Tools' },
    },
    {
      path: '/utilities',
      name: 'Utilities',
      component: () => import('./pages/Utilities/index.vue'),
      meta: { name: 'Utilities' },
    },
    {
      path: '/drivers',
      name: 'Drivers',
      component: () => import('./pages/Drivers/index.vue'),
      meta: { name: 'Drivers' },
    },
    {
      path: '/settings',
      name: 'Settings',
      component: () => import('./pages/Settings/index.vue'),
      meta: { name: 'Settings' },
    },
    { path: '/:pathMatch(.*)*', redirect: '/unisoc' },
  ],
})

export default router
