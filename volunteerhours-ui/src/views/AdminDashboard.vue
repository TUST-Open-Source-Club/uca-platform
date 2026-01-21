<script setup lang="ts">
import { reactive, ref } from 'vue'
import type { UploadFile } from 'element-plus'
import { createCompetition, createFormField, listCompetitions, listFormFields } from '../api/admin'
import { importStudents } from '../api/students'
import { useRequest } from '../composables/useRequest'

const competitionFormRef = ref()
const formFieldRef = ref()
const importFormRef = ref()
const competitions = ref('')
const formFields = ref('')
const importFile = ref<File | null>(null)
const result = ref('')
const competitionRequest = useRequest()
const formFieldRequest = useRequest()
const importRequest = useRequest()
const listRequest = useRequest()

const competitionForm = reactive({
  name: '',
})

const formField = reactive({
  form_type: 'volunteer',
  field_key: '',
  label: '',
  field_type: 'text',
  required: false,
  order_index: 1,
})

const importForm = reactive({
  fileName: '',
})

const competitionRules = {
  name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
}

const formFieldRules = {
  field_key: [{ required: true, message: '请输入字段 Key', trigger: 'blur' }],
  label: [{ required: true, message: '请输入字段标签', trigger: 'blur' }],
  form_type: [{ required: true, message: '请选择表单类型', trigger: 'change' }],
  field_type: [{ required: true, message: '请选择字段类型', trigger: 'change' }],
  order_index: [{ required: true, message: '请输入排序序号', trigger: 'change' }],
}

const importRules = {
  fileName: [{ required: true, message: '请选择 Excel 文件', trigger: 'change' }],
}

const loadCompetitions = async () => {
  await listRequest.run(async () => {
    const data = await listCompetitions()
    competitions.value = JSON.stringify(data, null, 2)
  })
}

const handleCompetitionCreate = async () => {
  if (!competitionFormRef.value) return
  await competitionFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await competitionRequest.run(
      async () => {
        const data = await createCompetition(competitionForm.name)
        result.value = JSON.stringify(data, null, 2)
        await loadCompetitions()
      },
      { successMessage: '已新增竞赛' },
    )
  })
}

const loadFormFields = async () => {
  await listRequest.run(async () => {
    const data = await listFormFields()
    formFields.value = JSON.stringify(data, null, 2)
  })
}

const handleFormFieldCreate = async () => {
  if (!formFieldRef.value) return
  await formFieldRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await formFieldRequest.run(
      async () => {
        const data = await createFormField(formField)
        result.value = JSON.stringify(data, null, 2)
        await loadFormFields()
      },
      { successMessage: '已新增字段' },
    )
  })
}

const handleImport = async () => {
  if (!importFormRef.value) return
  await importFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await importRequest.run(
      async () => {
        const data = await importStudents(importFile.value as File)
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已上传学生名单' },
    )
  })
}

const handleFileChange = (file: UploadFile) => {
  importFile.value = file.raw ?? null
  importForm.fileName = file.name ?? ''
}
</script>

<template>
  <section class="hero">
    <h1>管理台</h1>
    <p>维护学生名单、竞赛名称库与模板配置。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>学生名单导入</h3>
      <el-form ref="importFormRef" :model="importForm" :rules="importRules" label-position="top">
        <el-form-item label="Excel 文件" prop="fileName">
          <el-upload
            :auto-upload="false"
            :limit="1"
            :show-file-list="true"
            :on-change="handleFileChange"
          >
            <el-button>选择文件</el-button>
          </el-upload>
        </el-form-item>
        <el-button
          type="primary"
          style="margin-top: 12px"
          :loading="importRequest.loading"
          @click="handleImport"
        >
          上传名单
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛名称库</h3>
      <el-form ref="competitionFormRef" :model="competitionForm" :rules="competitionRules" label-position="top">
        <el-form-item label="竞赛名称" prop="name">
          <el-input v-model="competitionForm.name" placeholder="竞赛全称" />
        </el-form-item>
        <el-button type="primary" :loading="competitionRequest.loading" @click="handleCompetitionCreate">
          新增竞赛
        </el-button>
        <el-button style="margin-left: 8px" :loading="listRequest.loading" @click="loadCompetitions">
          刷新列表
        </el-button>
      </el-form>
      <pre v-if="competitions">{{ competitions }}</pre>
    </el-card>

    <el-card class="card">
      <h3>模板配置</h3>
      <el-form ref="formFieldRef" :model="formField" :rules="formFieldRules" label-position="top">
        <el-form-item label="字段 Key" prop="field_key">
          <el-input v-model="formField.field_key" placeholder="location" />
        </el-form-item>
        <el-form-item label="字段标签" prop="label">
          <el-input v-model="formField.label" placeholder="地点" />
        </el-form-item>
        <el-form-item label="表单类型" prop="form_type">
          <el-select v-model="formField.form_type">
            <el-option label="志愿服务" value="volunteer" />
            <el-option label="竞赛获奖" value="contest" />
          </el-select>
        </el-form-item>
        <el-form-item label="字段类型" prop="field_type">
          <el-select v-model="formField.field_type">
            <el-option label="文本" value="text" />
            <el-option label="数字" value="number" />
          </el-select>
        </el-form-item>
        <el-form-item label="是否必填">
          <el-select v-model="formField.required">
            <el-option label="必填" :value="true" />
            <el-option label="可选" :value="false" />
          </el-select>
        </el-form-item>
        <el-form-item label="排序序号" prop="order_index">
          <el-input-number v-model="formField.order_index" :min="1" />
        </el-form-item>
        <el-button type="primary" :loading="formFieldRequest.loading" @click="handleFormFieldCreate">
          新增字段
        </el-button>
        <el-button style="margin-left: 8px" :loading="listRequest.loading" @click="loadFormFields">
          刷新字段
        </el-button>
      </el-form>
      <pre v-if="formFields">{{ formFields }}</pre>
    </el-card>
  </div>

  <el-alert
    v-if="competitionRequest.error || formFieldRequest.error || importRequest.error || listRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="competitionRequest.error || formFieldRequest.error || importRequest.error || listRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
