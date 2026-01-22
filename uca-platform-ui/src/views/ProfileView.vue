<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { UploadFile } from 'element-plus'
import { logout as logoutRequest } from '../api/auth'
import { getSignatureProfile, uploadSignatureImage } from '../api/profile'
import { useRequest } from '../composables/useRequest'
import { useAuthStore } from '../stores/auth'

const signatureFile = ref<File | null>(null)
const profileRequest = useRequest()
const uploadRequest = useRequest()
const logoutState = useRequest()
const profile = ref<{ uploaded: boolean; signature_path?: string | null }>({ uploaded: false })
const router = useRouter()
const authStore = useAuthStore()

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

const handleLogout = async () => {
  await logoutState.run(async () => {
    await logoutRequest()
    authStore.logout()
    await router.push('/login')
  }, { successMessage: '已退出登录' })
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

  <el-card class="card">
    <h3>账号操作</h3>
    <p style="margin-bottom: 12px; color: var(--muted)">
      退出登录后需要重新完成认证。
    </p>
    <el-button type="danger" :loading="logoutState.loading" @click="handleLogout">
      退出登录
    </el-button>
  </el-card>

  <el-alert
    v-if="profileRequest.error || uploadRequest.error || logoutState.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="profileRequest.error || uploadRequest.error || logoutState.error"
    :closable="false"
  />
</template>
