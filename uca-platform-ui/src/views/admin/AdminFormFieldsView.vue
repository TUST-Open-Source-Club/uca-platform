<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import type { UploadFile } from 'element-plus'
import {
  createFormField,
  getExportTemplateFile,
  getLaborHourRules,
  listFormFields,
  updateLaborHourRules,
  uploadExportTemplateFile,
  type LaborHourRule,
} from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const activeTab = ref('fields')

const formFieldRef = ref()
const formFields = ref('')
const result = ref('')
const formFieldRequest = useRequest()
const listRequest = useRequest()

const formField = reactive({
  form_type: 'contest',
  field_key: '',
  label: '',
  field_type: 'text',
  required: false,
  order_index: 1,
})

const formFieldRules = {
  field_key: [{ required: true, message: '请输入字段 Key', trigger: 'blur' }],
  label: [{ required: true, message: '请输入字段标签', trigger: 'blur' }],
  form_type: [{ required: true, message: '请选择表单类型', trigger: 'change' }],
  field_type: [{ required: true, message: '请选择字段类型', trigger: 'change' }],
  order_index: [{ required: true, message: '请输入排序序号', trigger: 'change' }],
}

const exportTemplateFile = ref<File | null>(null)
const exportTemplateName = ref('')
const exportIssues = ref<string[]>([])
const exportOrientation = ref<'portrait' | 'landscape'>('portrait')
const exportRequest = useRequest()
const exportUploadRequest = useRequest()

const laborRules = reactive<LaborHourRule>({
  base_hours_a: 2,
  base_hours_b: 2,
  national_leader_hours: 4,
  national_member_hours: 2,
  provincial_leader_hours: 2,
  provincial_member_hours: 1,
  school_leader_hours: 1,
  school_member_hours: 1,
})
const laborRequest = useRequest()
const laborSaveRequest = useRequest()

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

const loadExportTemplate = async () => {
  await exportRequest.run(async () => {
    const data = await getExportTemplateFile('labor_hours')
    exportTemplateName.value = data.name || ''
    exportIssues.value = data.issues ?? []
    exportOrientation.value = data.orientation ?? 'portrait'
  })
}

const handleExportFileChange = (file: UploadFile) => {
  exportTemplateFile.value = file.raw ?? null
}

const handleExportUpload = async () => {
  if (!exportTemplateFile.value) {
    exportUploadRequest.error = '请选择 Excel 模板文件'
    return
  }
  await exportUploadRequest.run(
    async () => {
      const data = await uploadExportTemplateFile(
        'labor_hours',
        exportTemplateFile.value as File,
        exportOrientation.value,
      )
      exportTemplateName.value = data.name || ''
      exportIssues.value = data.issues ?? []
      exportOrientation.value = data.orientation ?? exportOrientation.value
    },
    { successMessage: '导出模板已更新' },
  )
}

const loadLaborRules = async () => {
  await laborRequest.run(async () => {
    const data = await getLaborHourRules()
    Object.assign(laborRules, data)
  })
}

const handleSaveLaborRules = async () => {
  await laborSaveRequest.run(
    async () => {
      const updated = await updateLaborHourRules({ ...laborRules })
      Object.assign(laborRules, updated)
    },
    { successMessage: '学时规则已保存' },
  )
}

onMounted(() => {
  void loadFormFields()
  void loadExportTemplate()
  void loadLaborRules()
})
</script>

<template>
  <section class="hero">
    <h1>模板与规则配置</h1>
    <p>维护表单字段、导出模板与劳动学时规则。</p>
  </section>

  <el-tabs v-model="activeTab" type="card">
    <el-tab-pane label="表单字段" name="fields">
      <div class="card-grid">
        <el-card class="card">
          <h3>新增字段</h3>
          <p style="margin-bottom: 12px; color: var(--muted)">
            字段用于学生填报与审核表格展示；字段 Key 将作为导入/导出映射的标识，请保持唯一且稳定。
          </p>
          <el-form ref="formFieldRef" :model="formField" :rules="formFieldRules" label-position="top">
            <el-form-item label="字段 Key" prop="field_key">
              <el-input v-model="formField.field_key" placeholder="location" />
            </el-form-item>
            <el-form-item label="字段标签" prop="label">
              <el-input v-model="formField.label" placeholder="地点" />
            </el-form-item>
            <el-form-item label="表单类型" prop="form_type">
              <el-select v-model="formField.form_type">
                <el-option label="竞赛获奖" value="contest" />
                <el-option label="劳动教育汇总Excel" value="labor_hours_excel" />
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
    </el-tab-pane>

    <el-tab-pane label="导出模板" name="export">
      <el-card class="card">
        <h3>上传劳动教育学时认定 PDF 模板</h3>
        <p>
          请上传包含占位符的 Excel 文件，后端将校验并用于导出 PDF。占位符规则请参考
          README 的“PDF 导出模板”说明。
        </p>
        <el-upload
          :auto-upload="false"
          :limit="1"
          :show-file-list="true"
          :on-change="handleExportFileChange"
        >
          <el-button>选择 Excel 模板</el-button>
        </el-upload>
        <el-form label-position="top" style="margin-top: 12px">
          <el-form-item label="纸张方向（A4）">
            <el-radio-group v-model="exportOrientation">
              <el-radio label="portrait">纵向</el-radio>
              <el-radio label="landscape">横向</el-radio>
            </el-radio-group>
          </el-form-item>
        </el-form>
        <el-button
          type="primary"
          style="margin-top: 12px"
          :loading="exportUploadRequest.loading"
          @click="handleExportUpload"
        >
          上传并校验
        </el-button>
        <div v-if="exportTemplateName" style="margin-top: 12px">
          <strong>当前模板：</strong>{{ exportTemplateName }}
        </div>
        <el-alert
          v-if="exportIssues.length"
          style="margin-top: 12px"
          type="error"
          show-icon
          :title="`模板校验发现 ${exportIssues.length} 个问题`"
          :closable="false"
        />
        <ul v-if="exportIssues.length" style="margin-top: 8px">
          <li v-for="(issue, index) in exportIssues" :key="index">{{ issue }}</li>
        </ul>
      </el-card>
    </el-tab-pane>

    <el-tab-pane label="学时规则" name="rules">
      <el-card class="card">
        <p style="margin-bottom: 12px; color: var(--muted)">
          学时规则用于自动计算推荐学时，最终学时以审核人员填写为准。
        </p>
        <el-form label-position="top" style="max-width: 360px">
          <el-form-item label="A 类基础学时">
            <el-input-number v-model="laborRules.base_hours_a" :min="0" />
          </el-form-item>
          <el-form-item label="B 类基础学时">
            <el-input-number v-model="laborRules.base_hours_b" :min="0" />
          </el-form-item>
          <el-form-item label="国家级负责人学时">
            <el-input-number v-model="laborRules.national_leader_hours" :min="0" />
          </el-form-item>
          <el-form-item label="国家级成员学时">
            <el-input-number v-model="laborRules.national_member_hours" :min="0" />
          </el-form-item>
          <el-form-item label="省级负责人学时">
            <el-input-number v-model="laborRules.provincial_leader_hours" :min="0" />
          </el-form-item>
          <el-form-item label="省级成员学时">
            <el-input-number v-model="laborRules.provincial_member_hours" :min="0" />
          </el-form-item>
          <el-form-item label="校级负责人学时">
            <el-input-number v-model="laborRules.school_leader_hours" :min="0" />
          </el-form-item>
          <el-form-item label="校级成员学时">
            <el-input-number v-model="laborRules.school_member_hours" :min="0" />
          </el-form-item>
          <el-button type="primary" :loading="laborSaveRequest.loading" @click="handleSaveLaborRules">
            保存学时规则
          </el-button>
        </el-form>
      </el-card>
    </el-tab-pane>
  </el-tabs>

  <el-alert
    v-if="
      formFieldRequest.error ||
      listRequest.error ||
      exportRequest.error ||
      exportUploadRequest.error ||
      laborRequest.error ||
      laborSaveRequest.error
    "
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="
      formFieldRequest.error ||
      listRequest.error ||
      exportRequest.error ||
      exportUploadRequest.error ||
      laborRequest.error ||
      laborSaveRequest.error
    "
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
