<script setup lang="ts">
import { ref } from 'vue'
import type { UploadFile } from 'element-plus'
import { reviewContest, reviewVolunteer } from '../api/records'
import { uploadSignature } from '../api/attachments'
import { useRequest } from '../composables/useRequest'

const formRef = ref()
const form = ref({
  recordType: 'volunteer',
  recordId: '',
  stage: 'first',
  hours: 0,
  status: 'approved',
  rejectionReason: '',
})
const signatureFile = ref<File | null>(null)
const result = ref('')
const reviewRequest = useRequest()
const signatureRequest = useRequest()

const validateHours = (_: unknown, value: number, callback: (error?: Error) => void) => {
  if (Number(value) < 0) {
    callback(new Error('学时不能为负数'))
    return
  }
  callback()
}

const validateRejection = (_: unknown, value: string, callback: (error?: Error) => void) => {
  if (form.value.status === 'rejected' && !value) {
    callback(new Error('请输入不通过原因'))
    return
  }
  callback()
}

const rules = {
  recordId: [{ required: true, message: '请输入记录 ID', trigger: 'blur' }],
  hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
  rejectionReason: [{ validator: validateRejection, trigger: 'blur' }],
}

const handleReview = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await reviewRequest.run(
      async () => {
        const payload = {
          stage: form.value.stage,
          hours: Number(form.value.hours),
          status: form.value.status,
          rejection_reason: form.value.rejectionReason || null,
        }
        const data =
          form.value.recordType === 'volunteer'
            ? await reviewVolunteer(form.value.recordId, payload)
            : await reviewContest(form.value.recordId, payload)
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '审核已提交' },
    )
  })
}

const handleSignatureUpload = async () => {
  if (!form.value.recordId) {
    signatureRequest.error = '请先填写记录 ID'
    return
  }
  if (!signatureFile.value) {
    signatureRequest.error = '请选择签名文件'
    return
  }
  result.value = ''
  await signatureRequest.run(
    async () => {
      const data = await uploadSignature(
        form.value.recordType,
        form.value.recordId,
        form.value.stage,
        signatureFile.value as File,
      )
      result.value = JSON.stringify(data, null, 2)
    },
    { successMessage: '签名已上传' },
  )
}

const handleFileChange = (file: UploadFile) => {
  signatureFile.value = file.raw ?? null
}
</script>

<template>
  <section class="hero">
    <h1>审核中心</h1>
    <p>初审与复审队列集中管理。</p>
  </section>

  <el-card class="card">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="记录类型">
        <el-select v-model="form.recordType">
          <el-option label="志愿服务" value="volunteer" />
          <el-option label="竞赛获奖" value="contest" />
        </el-select>
      </el-form-item>
      <el-form-item label="记录 ID" prop="recordId">
        <el-input v-model="form.recordId" placeholder="记录 UUID" />
      </el-form-item>
      <el-form-item label="审核阶段">
        <el-select v-model="form.stage">
          <el-option label="初审" value="first" />
          <el-option label="复审" value="final" />
        </el-select>
      </el-form-item>
      <el-form-item label="学时" prop="hours">
        <el-input-number v-model="form.hours" :min="0" />
      </el-form-item>
      <el-form-item label="状态">
        <el-select v-model="form.status">
          <el-option label="通过" value="approved" />
          <el-option label="不通过" value="rejected" />
        </el-select>
      </el-form-item>
      <el-form-item label="不通过原因" prop="rejectionReason">
        <el-input v-model="form.rejectionReason" placeholder="原因" />
      </el-form-item>
      <el-button type="primary" :loading="reviewRequest.loading" @click="handleReview">
        提交审核
      </el-button>
    </el-form>
  </el-card>

  <el-card class="card" style="margin-top: 20px">
    <h3>上传审核签名</h3>
    <el-upload
      :auto-upload="false"
      :limit="1"
      :show-file-list="true"
      :on-change="handleFileChange"
    >
      <el-button>选择文件</el-button>
    </el-upload>
    <el-button
      type="primary"
      style="margin-top: 12px"
      :loading="signatureRequest.loading"
      @click="handleSignatureUpload"
    >
      上传签名
    </el-button>
  </el-card>

  <el-alert
    v-if="reviewRequest.error || signatureRequest.error"
    class="card"
    style="margin-top: 20px"
    type="error"
    show-icon
    :title="reviewRequest.error || signatureRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 20px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
