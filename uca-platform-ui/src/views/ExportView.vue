<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { exportLaborHoursPdf, exportLaborHoursSummaryExcel, exportRecordPdf, exportStudent, exportSummary } from '../api/exports'
import { queryContest, type ContestRecord } from '../api/records'
import { useRequest } from '../composables/useRequest'

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

const pagination = reactive({
  page: 1,
  pageSize: 10,
})

const drawerVisible = ref(false)
const currentRecord = ref<ContestRecord | null>(null)
const currentIndex = ref(-1)

const summaryForm = reactive({
  department: '',
  major: '',
  className: '',
})
const studentForm = reactive({
  studentNo: '',
})
const laborForm = reactive({
  studentNo: '',
})

const laborSummaryForm = reactive({
  department: '',
  major: '',
  className: '',
})

const studentFormRef = ref()
const laborFormRef = ref()

const listRequest = useRequest()
const exportRequest = useRequest()
const summaryRequest = useRequest()
const studentRequest = useRequest()
const laborRequest = useRequest()
const laborSummaryRequest = useRequest()

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

const openDrawer = (record: ContestRecord) => {
  currentRecord.value = record
  currentIndex.value = filteredRecords.value.findIndex((item) => item.id === record.id)
  drawerVisible.value = true
}

const handlePrevNext = (direction: 'prev' | 'next') => {
  const list = filteredRecords.value
  if (!list.length || currentIndex.value === -1) return
  const nextIndex = direction === 'prev' ? currentIndex.value - 1 : currentIndex.value + 1
  const target = list[nextIndex]
  if (!target) return
  openDrawer(target)
}

const handleSelectionChange = (rows: ContestRecord[]) => {
  selection.value = rows
}

const handleToggleAll = () => {
  tableRef.value?.toggleAllSelection?.()
}

const handleExportSelected = async () => {
  if (!selection.value.length) return
  const confirmed = await ElMessageBox.confirm(
    `确认导出 ${selection.value.length} 条记录的 PDF？`,
    '批量导出',
    { type: 'info', confirmButtonText: '导出', cancelButtonText: '取消' },
  ).then(() => true).catch(() => false)
  if (!confirmed) return
  await exportRequest.run(async () => {
    for (const record of selection.value) {
      await exportRecordPdf('contest', record.id)
    }
  }, { successMessage: '导出任务已触发' })
}

const handleExportSingle = async () => {
  if (!currentRecord.value) return
  await exportRequest.run(async () => {
    await exportRecordPdf('contest', currentRecord.value!.id)
  }, { successMessage: '记录 PDF 已导出' })
}

const studentRules = {
  studentNo: [{ required: true, message: '请输入学号', trigger: 'blur' }],
}

const laborRules = {
  studentNo: [{ required: true, message: '请输入学号', trigger: 'blur' }],
}

const handleSummaryExport = async () => {
  await summaryRequest.run(
    async () => {
      await exportSummary({
        department: summaryForm.department || undefined,
        major: summaryForm.major || undefined,
        class_name: summaryForm.className || undefined,
      })
    },
    { successMessage: '汇总表已导出' },
  )
}

const handleStudentExport = async () => {
  if (!studentFormRef.value) return
  await studentFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await studentRequest.run(async () => {
      await exportStudent(studentForm.studentNo)
    }, { successMessage: '个人表已导出' })
  })
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

onMounted(() => {
  void loadRecords()
})
</script>

<template>
  <section class="hero">
    <h1>导出中心</h1>
    <p>通过表格选择记录导出 PDF，支持筛选、批量导出与分页。</p>
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
      <el-form-item label="竞赛级别">
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
      <el-button @click="handleToggleAll">全选</el-button>
      <el-button type="primary" :disabled="!selection.length" :loading="exportRequest.loading" @click="handleExportSelected">
        导出选中
      </el-button>
    </div>

    <el-table
      ref="tableRef"
      :data="pagedRecords"
      style="margin-top: 16px"
      @selection-change="handleSelectionChange"
      @row-dblclick="openDrawer"
    >
      <el-table-column type="selection" width="48" />
      <el-table-column prop="contest_name" label="竞赛名称" min-width="200" />
      <el-table-column prop="contest_year" label="年份" width="120" />
      <el-table-column prop="contest_category" label="类型" width="100" />
      <el-table-column prop="contest_level" label="级别" width="120" />
      <el-table-column prop="contest_role" label="角色" width="120" />
      <el-table-column prop="award_level" label="获奖等级" width="120" />
      <el-table-column prop="status" label="审核状态" width="140" />
      <el-table-column prop="match_status" label="匹配状态" width="120" />
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button size="small" @click="openDrawer(row)">详情</el-button>
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

    <el-empty v-if="!filteredRecords.length" description="暂无可导出记录" />
  </el-card>

  <el-drawer v-model="drawerVisible" size="480px" title="记录详情">
    <div v-if="currentRecord">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="竞赛名称">{{ currentRecord.contest_name }}</el-descriptions-item>
        <el-descriptions-item label="竞赛年份">{{ currentRecord.contest_year ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="竞赛类型">{{ currentRecord.contest_category ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="竞赛级别">{{ currentRecord.contest_level ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="竞赛角色">{{ currentRecord.contest_role ?? '-' }}</el-descriptions-item>
        <el-descriptions-item label="获奖等级">{{ currentRecord.award_level }}</el-descriptions-item>
        <el-descriptions-item label="审核状态">{{ currentRecord.status }}</el-descriptions-item>
      </el-descriptions>
      <el-button
        type="primary"
        style="margin-top: 16px"
        :loading="exportRequest.loading"
        @click="handleExportSingle"
      >
        导出当前记录
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

  <el-card class="card" style="margin-top: 20px">
    <h3>其他导出</h3>
    <div class="card-grid">
      <el-card class="card">
        <h4>学院汇总表</h4>
        <el-form label-position="top">
          <el-form-item label="院系">
            <el-input v-model="summaryForm.department" placeholder="信息学院" />
          </el-form-item>
          <el-form-item label="专业">
            <el-input v-model="summaryForm.major" placeholder="软件工程" />
          </el-form-item>
          <el-form-item label="班级">
            <el-input v-model="summaryForm.className" placeholder="软工1班" />
          </el-form-item>
          <el-button type="primary" :loading="summaryRequest.loading" @click="handleSummaryExport">
            导出 Excel
          </el-button>
        </el-form>
      </el-card>

      <el-card class="card">
        <h4>劳动教育学时汇总表（Excel）</h4>
        <el-form label-position="top">
          <el-form-item label="院系">
            <el-input v-model="laborSummaryForm.department" placeholder="信息学院" />
          </el-form-item>
          <el-form-item label="专业">
            <el-input v-model="laborSummaryForm.major" placeholder="软件工程" />
          </el-form-item>
          <el-form-item label="班级">
            <el-input v-model="laborSummaryForm.className" placeholder="软工1班" />
          </el-form-item>
          <el-button type="primary" :loading="laborSummaryRequest.loading" @click="handleLaborSummaryExport">
            导出 Excel
          </el-button>
        </el-form>
      </el-card>

      <el-card class="card">
        <h4>个人专项表</h4>
        <el-form ref="studentFormRef" :model="studentForm" :rules="studentRules" label-position="top">
          <el-form-item label="学号" prop="studentNo">
            <el-input v-model="studentForm.studentNo" placeholder="2023001" />
          </el-form-item>
          <el-button type="primary" :loading="studentRequest.loading" @click="handleStudentExport">
            导出个人表
          </el-button>
        </el-form>
      </el-card>

      <el-card class="card">
        <h4>劳动教育学时认定表</h4>
        <el-form ref="laborFormRef" :model="laborForm" :rules="laborRules" label-position="top">
          <el-form-item label="学号" prop="studentNo">
            <el-input v-model="laborForm.studentNo" placeholder="2023001" />
          </el-form-item>
          <el-button type="primary" :loading="laborRequest.loading" @click="handleLaborExport">
            导出 PDF
          </el-button>
        </el-form>
      </el-card>
    </div>
  </el-card>

  <el-alert
    v-if="listRequest.error || exportRequest.error || summaryRequest.error || studentRequest.error || laborRequest.error || laborSummaryRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="listRequest.error || exportRequest.error || summaryRequest.error || studentRequest.error || laborRequest.error || laborSummaryRequest.error"
    :closable="false"
  />
</template>
