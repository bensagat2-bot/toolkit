<template lang="pug">
dt#about {{ $t('setting__about') }}
dd
  .p.small
    | V1Per Servicing Toolkit — Android servicing toolkit for MediaTek, Unisoc & utilities.
  .p.small
    | 本软件完全免费，代码已开源。开源地址：
    span.hover.underline(:aria-label="$t('setting__click_open')" @click="openUrl('https://github.com/coolishsec0175/toolkit#readme')") https://github.com/coolishsec0175/toolkit
  .p.small
    | 最新版下载地址：
    span.hover.underline(:aria-label="$t('setting__click_open')" @click="openUrl('https://github.com/coolishsec0175/toolkit/releases')") GitHub Releases
  .p.small
    strong 本软件没有客服
    | ，使用前请仔细阅读各工具的说明。
  .p.small
    | 仍有问题可到&nbsp;GitHub&nbsp;
    span.hover.underline(:aria-label="$t('setting__click_open')" @click="openUrl('https://github.com/coolishsec0175/toolkit/issues')") 提交&nbsp;Issue
    | 。
  br
  .p.small
    | 你已签署本软件的
    base-btn(min @click="handleShowPact") 许可协议
    | 。

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