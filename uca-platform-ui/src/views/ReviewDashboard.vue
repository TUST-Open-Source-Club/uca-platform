<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import type { UploadFile } from 'element-plus'
import { ElMessageBox } from 'element-plus'
import { apiUrl } from '../api/client'
import { reviewContest, queryContest, type ContestRecord } from '../api/records'
import { uploadSignature } from '../api/attachments'
import { useRequest } from '../composables/useRequest'
import { useAuthStore } from '../stores/auth'

const authStore = useAuthStore()

const tableRef = ref()
const records = ref<ContestRecord[]>([])
const selection = ref<ContestRecord[]>([])

const filterForm = reactive({
  contest_name: '',
  contest_year: '',
  contest_category: '',
  contest_level: '',
  contest_role: '',
  award_level: '',
  status: '',
  match_status: '',
  student_id: '',
})
const emptyFilterForm = {
  contest_name: '',
  contest_year: '',
  contest_category: '',
  contest_level: '',
  contest_role: '',
  award_level: '',
  status: '',
  match_status: '',
  student_id: '',
}

const pagination = reactive({
  page: 1,
  pageSize: 10,
})

const reviewDrawerVisible = ref(false)
const bulkDialogVisible = ref(false)
const currentRecord = ref<ContestRecord | null>(null)
const currentIndex = ref(-1)
const signatureFile = ref<File | null>(null)

const reviewFormRef = ref()
const bulkFormRef = ref()

const reviewForm = reactive({
  stage: 'first',
  hours: 0,
  status: 'approved',
  rejectionReason: '',
})

const bulkForm = reactive({
  stage: 'first',
  hours: 0,
  status: 'approved',
  rejectionReason: '',
})

const listRequest = useRequest()
const reviewRequest = useRequest()
const bulkRequest = useRequest()
const signatureRequest = useRequest()

const statusOptions = [
  { label: '已提交', value: 'submitted' },
  { label: '已初审', value: 'first_reviewed' },
  { label: '已复审', value: 'final_reviewed' },
  { label: '不通过', value: 'rejected' },
]

const matchOptions = [
  { label: '已匹配', value: 'matched' },
  { label: '未匹配', value: 'unmatched' },
]

const filteredRecords = computed(() => {
  const filters = { ...filterForm }
  return records.value.filter((record) => {
    if (filters.contest_name && !record.contest_name.includes(filters.contest_name)) return false
    if (filters.contest_year && String(record.contest_year ?? '') !== filters.contest_year) return false
    if (filters.contest_category && (record.contest_category ?? '') !== filters.contest_category) return false
    if (filters.contest_level && (record.contest_level ?? '') !== filters.contest_level) return false
    if (filters.contest_role && (record.contest_role ?? '') !== filters.contest_role) return false
    if (filters.award_level && !record.award_level.includes(filters.award_level)) return false
    if (filters.status && record.status !== filters.status) return false
    if (filters.match_status && record.match_status !== filters.match_status) return false
    if (filters.student_id && record.student_id !== filters.student_id) return false
    return true
  })
})

const pagedRecords = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  return filteredRecords.value.slice(start, start + pagination.pageSize)
})

watch(
  () => [
    filterForm.contest_name,
    filterForm.contest_year,
    filterForm.contest_category,
    filterForm.contest_level,
    filterForm.contest_role,
    filterForm.award_level,
    filterForm.status,
    filterForm.match_status,
    filterForm.student_id,
  ],
  () => {
    pagination.page = 1
  },
)

const loadRecords = async () => {
  await listRequest.run(async () => {
    records.value = await queryContest()
  })
}

const clearFilters = () => {
  Object.assign(filterForm, emptyFilterForm)
  pagination.page = 1
}

const openReview = (record: ContestRecord) => {
  currentRecord.value = record
  currentIndex.value = filteredRecords.value.findIndex((item) => item.id === record.id)
  reviewForm.stage = authStore.user?.role === 'teacher' ? 'final' : 'first'
  reviewForm.status = record.status === 'rejected' ? 'rejected' : 'approved'
  reviewForm.rejectionReason = record.rejection_reason ?? ''
  if (reviewForm.stage === 'final') {
    reviewForm.hours = record.final_review_hours ?? record.first_review_hours ?? record.recommended_hours
  } else {
    reviewForm.hours = record.first_review_hours ?? record.recommended_hours
  }
  signatureFile.value = null
  reviewDrawerVisible.value = true
}

const handlePrevNext = (direction: 'prev' | 'next') => {
  const list = filteredRecords.value
  if (!list.length || currentIndex.value === -1) return
  const nextIndex = direction === 'prev' ? currentIndex.value - 1 : currentIndex.value + 1
  const target = list[nextIndex]
  if (!target) return
  openReview(target)
}

const validateHours = (_: unknown, value: number, callback: (error?: Error) => void) => {
  if (Number(value) < 0) {
    callback(new Error('学时不能为负数'))
    return
  }
  callback()
}

const validateRejection = (_: unknown, value: string, callback: (error?: Error) => void) => {
  if (reviewForm.status === 'rejected' && !value) {
    callback(new Error('请输入不通过原因'))
    return
  }
  callback()
}

const validateBulkRejection = (_: unknown, value: string, callback: (error?: Error) => void) => {
  if (bulkForm.status === 'rejected' && !value) {
    callback(new Error('请输入不通过原因'))
    return
  }
  callback()
}

const rules = {
  hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
  rejectionReason: [{ validator: validateRejection, trigger: 'blur' }],
}

const bulkRules = {
  hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
  rejectionReason: [{ validator: validateBulkRejection, trigger: 'blur' }],
}

const handleReview = async () => {
  if (!reviewFormRef.value || !currentRecord.value) return
  await reviewFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await reviewRequest.run(async () => {
      await reviewContest(currentRecord.value!.id, {
        stage: reviewForm.stage,
        hours: Number(reviewForm.hours),
        status: reviewForm.status,
        rejection_reason: reviewForm.rejectionReason || null,
      })
      await loadRecords()
    }, { successMessage: '审核已提交' })
  })
}

const handleBulkReview = async () => {
  if (!bulkFormRef.value || !selection.value.length) return
  await bulkFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    const confirmed = await ElMessageBox.confirm(
      `确认批量审核 ${selection.value.length} 条记录？`,
      '批量审核',
      { type: 'warning', confirmButtonText: '提交', cancelButtonText: '取消' },
    ).then(() => true).catch(() => false)
    if (!confirmed) return
    await bulkRequest.run(async () => {
      for (const record of selection.value) {
        await reviewContest(record.id, {
          stage: bulkForm.stage,
          hours: Number(bulkForm.hours),
          status: bulkForm.status,
          rejection_reason: bulkForm.rejectionReason || null,
        })
      }
      bulkDialogVisible.value = false
      selection.value = []
      await loadRecords()
    }, { successMessage: '批量审核已提交' })
  })
}

const handleSignatureUpload = async () => {
  if (!currentRecord.value) return
  if (!signatureFile.value) {
    signatureRequest.error = '请选择签名文件'
    return
  }
  await signatureRequest.run(async () => {
    await uploadSignature(
      'contest',
      currentRecord.value!.id,
      reviewForm.stage,
      signatureFile.value as File,
    )
  }, { successMessage: '签名已上传' })
}

const handleFileChange = (file: UploadFile) => {
  signatureFile.value = file.raw ?? null
}

const handleSelectionChange = (rows: ContestRecord[]) => {
  selection.value = rows
}

const handleToggleAll = () => {
  tableRef.value?.toggleAllSelection?.()
}

const resolveAttachmentUrl = (path: string) => apiUrl(path)

onMounted(async () => {
  await authStore.ensureSession()
  await loadRecords()
})
</script>

<template>
  <section class="hero">
    <h1>审核中心</h1>
    <p>从表格中双击记录进入审核，支持批量操作与分页筛选。</p>
  </section>

  <el-card class="card">
    <el-form label-position="top" style="display: flex; flex-wrap: wrap; gap: 12px">
      <el-form-item label="竞赛名称">
        <el-input v-model="filterForm.contest_name" placeholder="关键字" />
      </el-form-item>
      <el-form-item label="竞赛年份">
        <el-input v-model="filterForm.contest_year" placeholder="2024" />
      </el-form-item>
      <el-form-item label="竞赛类型">
        <el-select v-model="filterForm.contest_category" clearable placeholder="A/B">
          <el-option label="A 类" value="A" />
          <el-option label="B 类" value="B" />
        </el-select>
      </el-form-item>
      <el-form-item label="获奖级别">
        <el-input v-model="filterForm.contest_level" placeholder="国家级" />
      </el-form-item>
      <el-form-item label="竞赛角色">
        <el-input v-model="filterForm.contest_role" placeholder="负责人" />
      </el-form-item>
      <el-form-item label="获奖等级">
        <el-input v-model="filterForm.award_level" placeholder="一等奖" />
      </el-form-item>
      <el-form-item label="审核状态">
        <el-select v-model="filterForm.status" clearable>
          <el-option v-for="item in statusOptions" :key="item.value" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="竞赛匹配">
        <el-select v-model="filterForm.match_status" clearable>
          <el-option v-for="item in matchOptions" :key="item.value" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="学生 ID">
        <el-input v-model="filterForm.student_id" placeholder="UUID" />
      </el-form-item>
    </el-form>

    <div style="margin-top: 8px; display: flex; gap: 8px; justify-content: flex-end">
      <el-button :loading="listRequest.loading" @click="loadRecords">刷新列表</el-button>
      <el-button @click="clearFilters">清空筛选</el-button>
      <el-button @click="handleToggleAll">全选</el-button>
      <el-button type="primary" :disabled="!selection.length" @click="bulkDialogVisible = true">
        批量审核
      </el-button>
    </div>

    <el-table
      ref="tableRef"
      :data="pagedRecords"
      style="margin-top: 16px"
      @selection-change="handleSelectionChange"
      @row-dblclick="openReview"
    >
      <el-table-column type="selection" width="48" />
      <el-table-column prop="student_no" label="学号" min-width="140" />
      <el-table-column prop="student_name" label="姓名" width="120" />
      <el-table-column prop="department" label="学院" min-width="140" />
      <el-table-column prop="major" label="专业" min-width="140" />
      <el-table-column prop="class_name" label="班级" min-width="140" />
      <el-table-column prop="contest_name" label="竞赛名称" min-width="200" />
      <el-table-column prop="contest_year" label="年份" width="120" />
      <el-table-column prop="contest_category" label="类型" width="100" />
      <el-table-column prop="contest_level" label="级别" width="120" />
      <el-table-column prop="contest_role" label="角色" width="120" />
      <el-table-column prop="award_level" label="获奖等级" width="120" />
      <el-table-column prop="self_hours" label="自评学时" width="120" />
      <el-table-column prop="recommended_hours" label="推荐学时" width="120" />
      <el-table-column prop="status" label="审核状态" width="140" />
      <el-table-column prop="match_status" label="匹配状态" width="120" />
      <el-table-column prop="rejection_reason" label="不通过原因" min-width="160" />
      <el-table-column label="附件" min-width="140">
        <template #default="{ row }">
          <span v-if="row.attachments?.length">{{ row.attachments.length }} 个</span>
          <span v-else>无</span>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button size="small" @click="openReview(row)">审核</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-if="filteredRecords.length"
      style="margin-top: 12px; justify-content: flex-end"
      layout="total, sizes, prev, pager, next"
      :total="filteredRecords.length"
      :page-size="pagination.pageSize"
      :current-page="pagination.page"
      @update:page-size="(size: number) => { pagination.pageSize = size; pagination.page = 1 }"
      @update:current-page="(page: number) => { pagination.page = page }"
    />

    <el-empty v-if="!filteredRecords.length" description="暂无待审核记录" />
  </el-card>

  <el-drawer v-model="reviewDrawerVisible" size="520px" title="审核记录">
    <div v-if="currentRecord">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="学号">{{ currentRecord.student_no ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="姓名">{{ currentRecord.student_name ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="学院">{{ currentRecord.department ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="专业">{{ currentRecord.major ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="班级">{{ currentRecord.class_name ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="竞赛名称">{{ currentRecord.contest_name }}</el-descriptions-item>
        <el-descriptions-item label="竞赛年份">{{ currentRecord.contest_year ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="竞赛类型">{{ currentRecord.contest_category ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="获奖级别">{{ currentRecord.contest_level ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="竞赛角色">{{ currentRecord.contest_role ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="获奖等级">{{ currentRecord.award_level }}</el-descriptions-item>
        <el-descriptions-item label="推荐学时">{{ currentRecord.recommended_hours }}</el-descriptions-item>
      </el-descriptions>

      <el-divider />
      <h4>附件</h4>
      <div v-if="currentRecord.attachments?.length" style="display: grid; gap: 12px">
        <div
          v-for="attachment in currentRecord.attachments"
          :key="attachment.id"
          style="display: flex; flex-direction: column; gap: 6px"
        >
          <strong>{{ attachment.original_name }}</strong>
          <div v-if="attachment.mime_type.startsWith('image/')">
            <el-image
              :src="resolveAttachmentUrl(attachment.download_url)"
              style="width: 240px; max-height: 180px"
              fit="contain"
            />
          </div>
          <div v-else>
            <el-link :href="resolveAttachmentUrl(attachment.download_url)" target="_blank">
              下载/查看
            </el-link>
          </div>
        </div>
      </div>
      <el-empty v-else description="暂无附件" />

      <el-form ref="reviewFormRef" :model="reviewForm" :rules="rules" label-position="top" style="margin-top: 16px">
        <el-form-item label="审核阶段">
          <el-select v-model="reviewForm.stage">
            <el-option label="初审" value="first" />
            <el-option label="复审" value="final" />
          </el-select>
        </el-form-item>
        <el-form-item label="学时" prop="hours">
          <el-input-number v-model="reviewForm.hours" :min="0" />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="reviewForm.status">
            <el-option label="通过" value="approved" />
            <el-option label="不通过" value="rejected" />
          </el-select>
        </el-form-item>
        <el-form-item label="不通过原因" prop="rejectionReason">
          <el-input v-model="reviewForm.rejectionReason" placeholder="原因" />
        </el-form-item>
        <el-button type="primary" :loading="reviewRequest.loading" @click="handleReview">提交审核</el-button>
      </el-form>

      <el-divider />

      <h4>上传审核签名</h4>
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

      <div style="margin-top: 16px; display: flex; gap: 8px">
        <el-button :disabled="currentIndex <= 0" @click="handlePrevNext('prev')">上一条</el-button>
        <el-button
          :disabled="currentIndex === -1 || currentIndex >= filteredRecords.length - 1"
          @click="handlePrevNext('next')"
        >
          下一条
        </el-button>
      </div>
    </div>
  </el-drawer>

  <el-dialog v-model="bulkDialogVisible" title="批量审核" width="480px">
    <el-form ref="bulkFormRef" :model="bulkForm" :rules="bulkRules" label-position="top">
      <el-form-item label="审核阶段">
        <el-select v-model="bulkForm.stage">
          <el-option label="初审" value="first" />
          <el-option label="复审" value="final" />
        </el-select>
      </el-form-item>
      <el-form-item label="学时" prop="hours">
        <el-input-number v-model="bulkForm.hours" :min="0" />
      </el-form-item>
      <el-form-item label="状态">
        <el-select v-model="bulkForm.status">
          <el-option label="通过" value="approved" />
          <el-option label="不通过" value="rejected" />
        </el-select>
      </el-form-item>
      <el-form-item label="不通过原因" prop="rejectionReason">
        <el-input v-model="bulkForm.rejectionReason" placeholder="原因" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="bulkDialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="bulkRequest.loading" @click="handleBulkReview">提交</el-button>
    </template>
  </el-dialog>

  <el-alert
    v-if="listRequest.error || reviewRequest.error || bulkRequest.error || signatureRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="listRequest.error || reviewRequest.error || bulkRequest.error || signatureRequest.error"
    :closable="false"
  />
</template>
