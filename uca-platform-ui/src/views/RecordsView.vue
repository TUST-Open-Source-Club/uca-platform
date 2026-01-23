<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import type { UploadFile } from 'element-plus'
import { uploadContestAttachment } from '../api/attachments'
import { queryContest } from '../api/records'
import { useRequest } from '../composables/useRequest'
import { formatStatus } from '../utils/status'

const status = ref('')
const contest = ref<any[]>([])
const request = useRequest()
const uploadLoading = ref<Record<string, boolean>>({})

const handleLoad = async () => {
  await request.run(
    async () => {
      const contestData = await queryContest(status.value || undefined)
      contest.value = contestData as any[]
    },
    { successMessage: '已加载记录' },
  )
}

const handleAttachmentChange = async (recordId: string, file: UploadFile) => {
  if (!file.raw) return
  uploadLoading.value[recordId] = true
  try {
    await uploadContestAttachment(recordId, file.raw)
    ElMessage.success('附件已上传')
  } finally {
    uploadLoading.value[recordId] = false
  }
}
</script>

<template>
  <section class="hero">
    <h1>我的记录</h1>
    <p>查看竞赛获奖审核进度与附件。</p>
  </section>

  <el-card class="card">
    <el-form label-position="top">
      <el-form-item label="状态筛选">
        <el-select v-model="status" placeholder="请选择状态" clearable>
          <el-option label="全部" value="" />
          <el-option label="已提交" value="submitted" />
          <el-option label="已初审" value="first_reviewed" />
          <el-option label="已复审" value="final_reviewed" />
          <el-option label="不通过" value="rejected" />
        </el-select>
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleLoad">加载记录</el-button>
    </el-form>
  </el-card>

  <el-alert
    v-if="request.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="request.error"
    :closable="false"
  />

  <div class="card-grid" style="margin-top: 20px">
    <el-card class="card">
      <h3>竞赛获奖记录</h3>
      <el-table v-if="contest.length" :data="contest">
        <el-table-column prop="contest_name" label="竞赛名称" />
        <el-table-column prop="contest_level" label="获奖级别" />
        <el-table-column prop="award_level" label="获奖等级" />
        <el-table-column label="状态">
          <template #default="{ row }">
            {{ formatStatus(row.status) }}
          </template>
        </el-table-column>
        <el-table-column label="附件上传" width="180">
          <template #default="{ row }">
            <el-upload
              :auto-upload="false"
              :limit="1"
              :show-file-list="false"
              :on-change="(file: UploadFile) => handleAttachmentChange(row.id, file)"
            >
              <el-button size="small" :loading="uploadLoading[row.id]">上传附件</el-button>
            </el-upload>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-else description="暂无记录" />
    </el-card>
  </div>
</template>
