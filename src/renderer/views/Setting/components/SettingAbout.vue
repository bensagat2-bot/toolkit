<template lang="pug">
dt#about {{ $t('setting__about') }}
dd
  .p.small
    | V1Per Servicing Toolkit — Android servicing toolkit for MediaTek, Unisoc & utilities.
  .p.small
    span.hover.underline(:aria-label="$t('setting__click_open')" @click="openUrl('https://github.com/coolishsec0175/toolkit#readme')") https://github.com/coolishsec0175/toolkit
  .p.small
    span.hover.underline(:aria-label="$t('setting__click_open')" @click="openUrl('https://github.com/coolishsec0175/toolkit/releases')") GitHub Releases
  .p.small
    span.hover.underline(:aria-label="$t('setting__click_open')" @click="openUrl('https://github.com/coolishsec0175/toolkit/issues')") Report Issues
  br
  .p.small
    | By:&nbsp;
    strong coolishsec0175
</template>

<script>
import { isShowPact } from '@renderer/store'
import { openUrl, clipboardWriteText } from '@common/utils/electron'

export default {
  name: 'SettingAbout',
  setup() {
    const handleShowPact = () => {
      isShowPact.value = true
    }
    return {
      openUrl,
      clipboardWriteText,
      handleShowPact,
    }
  },
}
</script>