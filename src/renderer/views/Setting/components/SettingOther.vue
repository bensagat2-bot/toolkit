<template lang="pug">
dt#other {{ $t('setting__other') }}
dd
  div
    .gap-top
      base-checkbox(id="setting_transparent_window" :model-value="appSetting['common.transparentWindow']" :label="$t('setting__other_transparent_window')" @update:model-value="updateSetting({'common.transparentWindow': $event})")
      svg-icon(class="help-icon" name="help-circle-outline" :aria-label="$t('setting__other_transparent_window_tip')")
dd
  h3#other_resource_cache {{ $t('setting__other_resource_cache') }}
  div
    .p
      | {{ $t('setting__other_resource_cache_label') }}
      span.auto-hidden {{ cacheSize }}
    .p
      base-btn.btn(min :disabled="isDisabledResourceCacheClear" @click="clearResourceCache") {{ $t('setting__other_resource_cache_clear_btn') }}

dd
  h3#other_other_source {{ $t('setting__other_other_cache') }}
  div
    .p
      base-btn.btn(min @click="handleClearListData") {{ $t('setting__other_listdata_clear_btn') }}
</template>

<script>
import { ref } from '@common/utils/vueTools'
import {
  clearCache, getCacheSize,
} from '@renderer/utils/ipc'
import { sizeFormate } from '@common/utils/common'
import { dialog } from '@renderer/plugins/Dialog'
import { useI18n } from '@renderer/plugins/i18n'
import { appSetting, updateSetting } from '@renderer/store/setting'

export default {
  name: 'SettingOther',
  setup() {
    const t = useI18n()

    const cacheSize = ref('0 B')
    const isDisabledResourceCacheClear = ref(false)
    const refreshCacheSize = () => {
      void getCacheSize().then(size => {
        cacheSize.value = sizeFormate(size)
      })
    }
    const clearResourceCache = async() => {
      if (!await dialog.confirm({
        message: t('setting__other_resource_cache_tip_confirm'),
        cancelButtonText: t('cancel_button_text'),
        confirmButtonText: t('setting__other_resource_cache_confirm'),
      })) return
      isDisabledResourceCacheClear.value = true
      void clearCache().then(() => {
        refreshCacheSize()
        isDisabledResourceCacheClear.value = false
      })
    }
    refreshCacheSize()

    const handleClearListData = async() => {
      if (!await dialog.confirm({
        message: t('setting__other_listdata_clear_tip_confirm'),
        cancelButtonText: t('cancel_button_text'),
        confirmButtonText: t('setting__other_resource_cache_confirm'),
      })) return
      console.log('clear list data')
    }

    return {
      appSetting,
      updateSetting,
      cacheSize,
      isDisabledResourceCacheClear,
      clearResourceCache,
      handleClearListData,
    }
  },
}
</script>