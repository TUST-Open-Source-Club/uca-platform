<script setup lang="ts">
import { reactive, ref } from 'vue'
import { exportLaborHoursPdf, exportRecordPdf, exportStudent, exportSummary } from '../api/exports'
import { useRequest } from '../composables/useRequest'

const summaryForm = reactive({
  department: '',
  major: '',
  className: '',
})
const studentForm = reactive({
  studentNo: '',
})
const recordForm = reactive({
  recordType: 'contest',
  recordId: '',
})

const laborForm = reactive({
  studentNo: '',
})

const studentFormRef = ref()
const recordFormRef = ref()
const laborFormRef = ref()
const summaryRequest = useRequest()
const studentRequest = useRequest()
const recordRequest = useRequest()
const laborRequest = useRequest()

const studentRules = {
  studentNo: [{ required: true, message: '请输入学号', trigger: 'blur' }],
}

const recordRules = {
  recordId: [{ required: true, message: '请输入记录 ID', trigger: 'blur' }],
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

const handleRecordExport = async () => {
  if (!recordFormRef.value) return
  await recordFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await recordRequest.run(async () => {
      await exportRecordPdf(recordForm.recordType, recordForm.recordId)
    }, { successMessage: '记录 PDF 已导出' })
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
</script>

<template>
  <section class="hero">
    <h1>导出中心</h1>
    <p>按学院、专业或个人导出 Excel/PDF。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>学院汇总表</h3>
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
      <h3>个人专项表</h3>
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
      <h3>记录 PDF</h3>
      <el-form ref="recordFormRef" :model="recordForm" :rules="recordRules" label-position="top">
        <el-form-item label="记录类型">
          <el-select v-model="recordForm.recordType">
            <el-option label="竞赛获奖" value="contest" />
          </el-select>
        </el-form-item>
        <el-form-item label="记录 ID" prop="recordId">
          <el-input v-model="recordForm.recordId" placeholder="记录 UUID" />
        </el-form-item>
        <el-button type="primary" :loading="recordRequest.loading" @click="handleRecordExport">
          导出 PDF
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>劳动教育学时认定表</h3>
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

  <el-alert
    v-if="summaryRequest.error || studentRequest.error || recordRequest.error || laborRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="summaryRequest.error || studentRequest.error || recordRequest.error || laborRequest.error"
    :closable="false"
  />
</template>
