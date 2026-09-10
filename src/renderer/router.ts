import { createRouter, createWebHashHistory } from 'vue-router'
import MediatekTools from './views/MediatekTools/index.vue'
import UnisocTools from './views/UnisocTools/index.vue'
import UtilitiesTools from './views/UtilitiesTools/index.vue'
import Drivers from './views/Drivers/index.vue'
import Setting from './views/Setting/index.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/mediatek',
    },
    {
      path: '/mediatek',
      name: 'MediatekTools',
      component: MediatekTools,
      meta: {
        name: 'Mediatek Tools',
      },
    },
    {
      path: '/unisoc',
      name: 'UnisocTools',
      component: UnisocTools,
      meta: {
        name: 'Unisoc Tools',
      },
    },
    {
      path: '/utilities',
      name: 'UtilitiesTools',
      component: UtilitiesTools,
      meta: {
        name: 'Utilities Tools',
      },
    },
    {
      path: '/drivers',
      name: 'Drivers',
      component: Drivers,
      meta: {
        name: 'Drivers',
      },
    },
    {
      path: '/setting',
      name: 'Setting',
      component: Setting,
      meta: {
        name: 'Setting',
      },
    },
    { path: '/:pathMatch(.*)*', redirect: '/mediatek' },
  ],
  linkActiveClass: 'active-link',
  linkExactActiveClass: 'exact-active-link',
})

export default router
