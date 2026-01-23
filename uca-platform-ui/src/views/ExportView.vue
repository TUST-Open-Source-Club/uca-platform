<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { apiUrl } from '../api/client'
import { exportLaborHoursPdf, exportLaborHoursSummaryExcel } from '../api/exports'
import { queryContest, type ContestRecord } from '../api/records'
import { useRequest } from '../composables/useRequest'

const laborForm = reactive({
  studentNo: '',
})

const laborSummaryForm = reactive({
  department: '',
  major: '',
  className: '',
})

const filterForm = reactive({
  student_no: '',
  student_name: '',
  department: '',
  major: '',
  class_name: '',
  contest_name: '',
  contest_year: '',
  contest_category: '',
  contest_level: '',
  award_level: '',
})
const emptyFilterForm = {
  student_no: '',
  student_name: '',
  department: '',
  major: '',
  class_name: '',
  contest_name: '',
  contest_year: '',
  contest_category: '',
  contest_level: '',
  award_level: '',
}

const pagination = reactive({
  page: 1,
  pageSize: 10,
})

const laborFormRef = ref()

const laborRequest = useRequest()
const laborSummaryRequest = useRequest()
const listRequest = useRequest()
const records = ref<ContestRecord[]>([])
const laborRules = {
  studentNo: [{ required: true, message: '请输入学号', trigger: 'blur' }],
}

const handleLaborExport = async () => {
  if (!laborFormRef.value) return
  await laborFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await laborRequest.run(async () => {
      await exportLaborHoursPdf(laborForm.studentNo)
    }, { successMessage: '劳动教育学时认定表已导出' })
  })
}

const handleLaborSummaryExport = async () => {
  await laborSummaryRequest.run(
    async () => {
      await exportLaborHoursSummaryExcel({
        department: laborSummaryForm.department || undefined,
        major: laborSummaryForm.major || undefined,
        class_name: laborSummaryForm.className || undefined,
      })
    },
    { successMessage: '劳动教育学时汇总表已导出' },
  )
}

const loadRecords = async () => {
  await listRequest.run(async () => {
    records.value = await queryContest()
  })
}

const resolveAttachmentUrl = (path: string) => apiUrl(path)

const filteredRecords = computed(() => {
  const filters = { ...filterForm }
  return records.value.filter((record) => {
    if (filters.student_no && !String(record.student_no ?? '').includes(filters.student_no)) return false
    if (filters.student_name && !String(record.student_name ?? '').includes(filters.student_name)) return false
    if (filters.department && !String(record.department ?? '').includes(filters.department)) return false
    if (filters.major && !String(record.major ?? '').includes(filters.major)) return false
    if (filters.class_name && !String(record.class_name ?? '').includes(filters.class_name)) return false
    if (filters.contest_name && !record.contest_name.includes(filters.contest_name)) return false
    if (filters.contest_year && String(record.contest_year ?? '') !== filters.contest_year) return false
    if (filters.contest_category && (record.contest_category ?? '') !== filters.contest_category) return false
    if (filters.contest_level && !String(record.contest_level ?? '').includes(filters.contest_level)) return false
    if (filters.award_level && !record.award_level.includes(filters.award_level)) return false
    return true
  })
})

const pagedRecords = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  return filteredRecords.value.slice(start, start + pagination.pageSize)
})

const clearFilters = () => {
  Object.assign(filterForm, emptyFilterForm)
  pagination.page = 1
}

watch(
  () => [
    filterForm.student_no,
    filterForm.student_name,
    filterForm.department,
    filterForm.major,
    filterForm.class_name,
    filterForm.contest_name,
    filterForm.contest_year,
    filterForm.contest_category,
    filterForm.contest_level,
    filterForm.award_level,
  ],
  () => {
    pagination.page = 1
  },
)

onMounted(async () => {
  await loadRecords()
})
</script>

<template>
  <section class="hero">
    <h1>导出中心</h1>
    <p>仅保留劳动教育学时汇总表（Excel）与劳动教育学时认定表（PDF）。</p>
  </section>

  <el-card class="card" style="margin-top: 20px">
    <h3>其他导出</h3>
    <div class="card-grid">
      <el-card class="card">
        <h4>劳动教育学时汇总表（Excel）</h4>
        <el-form label-position="top">
          <el-form-item label="院系">
            <el-input v-model="laborSummaryForm.department" placeholder="人工智能学院" />
          </el-form-item>
          <el-form-item label="专业">
            <el-input v-model="laborSummaryForm.major" placeholder="软件工程" />
          </el-form-item>
          <el-form-item label="班级">
            <el-input v-model="laborSummaryForm.className" placeholder="222014" />
          </el-form-item>
          <el-button type="primary" :loading="laborSummaryRequest.loading" @click="handleLaborSummaryExport">
            导出 Excel
          </el-button>
        </el-form>
      </el-card>

      <el-card class="card">
        <h4>劳动教育学时认定表</h4>
        <el-form ref="laborFormRef" :model="laborForm" :rules="laborRules" label-position="top">
          <el-form-item label="学号" prop="studentNo">
            <el-input v-model="laborForm.studentNo" placeholder="22201400" />
          </el-form-item>
          <el-button type="primary" :loading="laborRequest.loading" @click="handleLaborExport">
            导出 PDF
          </el-button>
        </el-form>
      </el-card>
    </div>
  </el-card>

  <el-card class="card" style="margin-top: 20px">
    <h3>竞赛记录预览</h3>
    <p style="margin-bottom: 12px">导出前可在此核对学生信息与附件。</p>
    <el-button :loading="listRequest.loading" @click="loadRecords">刷新列表</el-button>

    <el-form label-position="top" style="margin-top: 16px; display: flex; flex-wrap: wrap; gap: 12px">
      <el-form-item label="学号">
        <el-input v-model="filterForm.student_no" placeholder="2023001" />
      </el-form-item>
      <el-form-item label="姓名">
        <el-input v-model="filterForm.student_name" placeholder="张三" />
      </el-form-item>
      <el-form-item label="学院">
        <el-input v-model="filterForm.department" placeholder="人工智能学院" />
      </el-form-item>
      <el-form-item label="专业">
        <el-input v-model="filterForm.major" placeholder="软件工程" />
      </el-form-item>
      <el-form-item label="班级">
        <el-input v-model="filterForm.class_name" placeholder="232011" />
      </el-form-item>
      <el-form-item label="竞赛名称">
        <el-input v-model="filterForm.contest_name" placeholder="数学建模" />
      </el-form-item>
      <el-form-item label="年份">
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
      <el-form-item label="获奖等级">
        <el-input v-model="filterForm.award_level" placeholder="一等奖" />
      </el-form-item>
    </el-form>
    <div style="margin-top: 8px; display: flex; gap: 8px; justify-content: flex-end">
      <el-button @click="clearFilters">清空筛选</el-button>
    </div>

    <el-table v-if="filteredRecords.length" :data="pagedRecords" style="margin-top: 16px">
      <el-table-column type="expand">
        <template #default="{ row }">
          <div style="display: grid; gap: 12px">
            <strong>附件</strong>
            <div v-if="row.attachments?.length" style="display: grid; gap: 12px">
              <div
                v-for="attachment in row.attachments"
                :key="attachment.id"
                style="display: flex; flex-direction: column; gap: 6px"
              >
                <span>{{ attachment.original_name }}</span>
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
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="student_no" label="学号" min-width="140" />
      <el-table-column prop="student_name" label="姓名" width="120" />
      <el-table-column prop="department" label="学院" min-width="140" />
      <el-table-column prop="major" label="专业" min-width="140" />
      <el-table-column prop="class_name" label="班级" min-width="140" />
      <el-table-column prop="contest_name" label="竞赛名称" min-width="180" />
      <el-table-column prop="contest_year" label="年份" width="100" />
      <el-table-column prop="contest_level" label="获奖级别" width="120" />
      <el-table-column prop="award_level" label="获奖等级" width="120" />
      <el-table-column prop="self_hours" label="自评学时" width="120" />
      <el-table-column prop="recommended_hours" label="推荐学时" width="120" />
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
    <el-empty v-else description="暂无竞赛记录" />
  </el-card>

  <el-alert
    v-if="laborRequest.error || laborSummaryRequest.error || listRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="laborRequest.error || laborSummaryRequest.error || listRequest.error"
    :closable="false"
  />
</template>
