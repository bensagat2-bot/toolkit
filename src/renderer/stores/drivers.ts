import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { driverList, type DriverCategory } from '@/data/drivers'

export const useDriversStore = defineStore('drivers', () => {
  const query = ref('')
  const category = ref<DriverCategory | 'all'>('all')
  const sort = ref('recommended')
  const page = ref(1)
  const pageSize = 12

  const filtered = computed(() => {
    let list = driverList
    if (category.value !== 'all') list = list.filter(item => item.category === category.value)
    if (query.value) {
      const q = query.value.toLowerCase()
      list = list.filter(
        item => item.name.toLowerCase().includes(q) || item.description.toLowerCase().includes(q),
      )
    }
    if (sort.value === 'latest') list = [...list].reverse()
    if (sort.value === 'popular') list = [...list].sort((a, b) => b.rating - a.rating)
    return list
  })

  const total = computed(() => filtered.value.length)
  const maxPage = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))
  const pageList = computed(() =>
    filtered.value.slice((page.value - 1) * pageSize, page.value * pageSize),
  )
  const allNames = driverList.map(item => item.name)

  watch([query, category, sort], () => {
    page.value = 1
  })

  const setQuery = (value: string) => {
    query.value = value
  }

  const setCategory = (value: DriverCategory | 'all') => {
    category.value = value
  }

  const setSort = (value: string) => {
    sort.value = value
  }

  const setPage = (value: number) => {
    page.value = value
  }

  return {
    query,
    category,
    sort,
    page,
    pageSize,
    total,
    maxPage,
    pageList,
    allNames,
    setQuery,
    setCategory,
    setSort,
    setPage,
  }
})