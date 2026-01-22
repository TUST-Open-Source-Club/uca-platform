<script setup lang="ts">
import { onMounted, ref } from 'vue'
import type { UploadFile } from 'element-plus'
import { getSignatureProfile, uploadSignatureImage } from '../api/profile'
import { useRequest } from '../composables/useRequest'

const signatureFile = ref<File | null>(null)
const profileRequest = useRequest()
const uploadRequest = useRequest()
const profile = ref<{ uploaded: boolean; signature_path?: string | null }>({ uploaded: false })

const loadProfile = async () => {
  await profileRequest.run(async () => {
    profile.value = await getSignatureProfile()
  }, { silent: true })
}

const handleFileChange = (file: UploadFile) => {
  signatureFile.value = file.raw ?? null
}

const handleUpload = async () => {
  if (!signatureFile.value) {
    uploadRequest.error = '请选择签名图片'
    return
  }
  await uploadRequest.run(async () => {
    profile.value = await uploadSignatureImage(signatureFile.value as File)
    signatureFile.value = null
  }, { successMessage: '签名已更新' })
}

onMounted(() => {
  void loadProfile()
})
</script>

<template>
  <section class="hero">
    <h1>个人中心</h1>
    <p>上传审核签名图片，用于导出 PDF 中的电子签名展示。</p>
  </section>

  <el-card class="card">
    <h3>电子签名图片</h3>
    <p style="margin-bottom: 12px; color: var(--muted)">
      当前状态：{{ profile.uploaded ? '已上传' : '未上传' }}
    </p>
    <el-upload :auto-upload="false" :limit="1" :show-file-list="true" :on-change="handleFileChange">
      <el-button>选择签名图片</el-button>
    </el-upload>
    <el-button type="primary" style="margin-top: 12px" :loading="uploadRequest.loading" @click="handleUpload">
      上传签名
    </el-button>
  </el-card>

  <el-alert
    v-if="profileRequest.error || uploadRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="profileRequest.error || uploadRequest.error"
    :closable="false"
  />
</template>
