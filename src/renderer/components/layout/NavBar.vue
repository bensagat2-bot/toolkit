<template>
  <nav class="menu">
    <ul>
      <li v-for="item in menus" :key="item.to">
        <router-link
          :class="['link', { active: $route.name === item.name }]"
          :to="item.to"
          :aria-label="item.label"
          :title="item.label"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path v-for="(d, i) in item.paths" :key="i" :d="d" />
            <line v-for="(l, i) in item.lines" :key="'l' + i" v-bind="l" />
            <rect v-for="(r, i) in item.rects" :key="'r' + i" v-bind="r" />
            <circle v-for="(c, i) in item.circles" :key="'c' + i" v-bind="c" />
            <polyline v-for="(p, i) in item.polylines" :key="'p' + i" :points="p" />
          </svg>
        </router-link>
      </li>
    </ul>
  </nav>
</template>

<script setup lang="ts">
const menus: Array<{
  to: string
  name: string
  label: string
  paths: string[]
  lines: Array<Record<string, string>>
  rects: Array<Record<string, string>>
  circles: Array<Record<string, string>>
  polylines: string[]
}> = [
  {
    to: '/mediatek',
    name: 'MediaTek',
    label: 'MediaTek Tools',
    paths: [],
    lines: [
      { x1: '9', y1: '1', x2: '9', y2: '5' },
      { x1: '15', y1: '1', x2: '15', y2: '5' },
      { x1: '9', y1: '19', x2: '9', y2: '23' },
      { x1: '15', y1: '19', x2: '15', y2: '23' },
      { x1: '1', y1: '9', x2: '5', y2: '9' },
      { x1: '1', y1: '15', x2: '5', y2: '15' },
      { x1: '19', y1: '9', x2: '23', y2: '9' },
      { x1: '19', y1: '15', x2: '23', y2: '15' },
    ],
    rects: [
      { x: '5', y: '5', width: '14', height: '14', rx: '2' },
      { x: '9', y: '9', width: '6', height: '6' },
    ],
    circles: [],
    polylines: [],
  },
  {
    to: '/unisoc',
    name: 'Unisoc',
    label: 'Unisoc Tools',
    paths: [],
    lines: [
      { x1: '9', y1: '1', x2: '9', y2: '4' },
      { x1: '15', y1: '1', x2: '15', y2: '4' },
      { x1: '9', y1: '20', x2: '9', y2: '23' },
      { x1: '15', y1: '20', x2: '15', y2: '23' },
      { x1: '1', y1: '9', x2: '4', y2: '9' },
      { x1: '1', y1: '15', x2: '4', y2: '15' },
      { x1: '20', y1: '9', x2: '23', y2: '9' },
      { x1: '20', y1: '15', x2: '23', y2: '15' },
    ],
    rects: [
      { x: '4', y: '4', width: '16', height: '16', rx: '2' },
      { x: '9', y: '9', width: '6', height: '6' },
    ],
    circles: [],
    polylines: [],
  },
  {
    to: '/xiaomi',
    name: 'Xiaomi',
    label: 'Xiaomi Tools',
    paths: [],
    lines: [],
    rects: [
      { x: '5', y: '2', width: '14', height: '20', rx: '2' },
    ],
    circles: [
      { cx: '12', cy: '18', r: '1' },
    ],
    polylines: [],
  },
  {
    to: '/utilities',
    name: 'Utilities',
    label: 'Utilities',
    paths: ['M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z'],
    lines: [],
    rects: [],
    circles: [],
    polylines: [],
  },
  {
    to: '/drivers',
    name: 'Drivers',
    label: 'Drivers',
    paths: ['M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4'],
    lines: [
      { x1: '7', y1: '10', x2: '12', y2: '15' },
      { x1: '12', y1: '15', x2: '17', y2: '10' },
      { x1: '12', y1: '15', x2: '12', y2: '3' },
    ],
    rects: [],
    circles: [],
    polylines: [],
  },
  {
    to: '/settings',
    name: 'Settings',
    label: 'Settings',
    paths: [],
    lines: [
      { x1: '4', y1: '21', x2: '4', y2: '14' },
      { x1: '4', y1: '10', x2: '4', y2: '3' },
      { x1: '12', y1: '21', x2: '12', y2: '12' },
      { x1: '12', y1: '8', x2: '12', y2: '3' },
      { x1: '20', y1: '21', x2: '20', y2: '16' },
      { x1: '20', y1: '12', x2: '20', y2: '3' },
      { x1: '1', y1: '14', x2: '7', y2: '14' },
      { x1: '9', y1: '8', x2: '15', y2: '8' },
      { x1: '17', y1: '16', x2: '23', y2: '16' },
    ],
    rects: [],
    circles: [],
    polylines: [],
  },
]
</script>

<style scoped>
.menu {
  flex: auto;
  -webkit-app-region: no-drag;
}

ul {
  display: flex;
  flex-flow: column nowrap;
}

li {
  position: relative;
}

li::before {
  content: '';
  display: block;
  width: 100%;
  padding-bottom: 84%;
}

.link {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-nav-font);
  text-align: center;
  outline: none;
  transition: background-color 0.3s ease, opacity 0.3s ease;
}

.link::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  width: 3px;
  height: 100%;
  background-color: var(--color-primary-dark-200-alpha-700);
  border-radius: 4px;
  transform: translateX(-100%);
  transition: transform 0.3s ease;
}

.link svg {
  width: 32%;
  height: 32%;
}

.link:hover:not(.active) {
  opacity: 0.8;
  background-color: var(--color-primary-light-400-alpha-700);
}

.link:active:not(.active) {
  opacity: 0.6;
  background-color: var(--color-primary-light-300-alpha-600);
}

.link.active {
  background-color: var(--color-primary-light-300-alpha-700);
}

.link.active:hover {
  background-color: var(--color-primary-light-300-alpha-800);
}

.link.active::before {
  transform: translateX(0);
}
</style>