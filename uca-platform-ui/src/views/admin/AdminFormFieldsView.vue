<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import {
  createFormField,
  getExportTemplate,
  getLaborHourRules,
  listFormFields,
  listImportTemplates,
  updateExportTemplate,
  updateImportTemplate,
  updateLaborHourRules,
  type ExportTemplate,
  type ImportTemplate,
  type ImportTemplateField,
  type LaborHourRule,
} from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

type LayoutField = { key: string; label: string }
type LayoutSection =
  | { type: 'info'; title: string; fields: LayoutField[] }
  | { type: 'table'; title: string; columns: LayoutField[] }
type ExportLayout = {
  title: string
  sections: LayoutSection[]
  signature?: { first_label?: string; final_label?: string }
}

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

const importTemplates = ref<ImportTemplate[]>([])
const selectedImportKey = ref('competition_library')
const importRequest = useRequest()
const importSaveRequest = useRequest()
const importEdit = reactive<{ name: string; fields: ImportTemplateField[] }>({
  name: '',
  fields: [],
})

const exportRequest = useRequest()
const exportSaveRequest = useRequest()
const exportTemplate = ref<ExportTemplate | null>(null)
const exportTemplateName = ref('劳动教育学时认定表')
const exportLayout = reactive<ExportLayout>({
  title: '',
  sections: [],
  signature: { first_label: '初审教师签名', final_label: '复审教师签名' },
})

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

const importOptions = computed(() =>
  importTemplates.value.map((item) => ({
    label: item.name,
    value: item.template_key,
  })),
)

const templateKeysHint = [
  'student_no',
  'name',
  'gender',
  'department',
  'major',
  'class_name',
  'phone',
  'contest_year',
  'contest_category',
  'contest_name',
  'contest_level',
  'contest_role',
  'award_level',
  'award_date',
  'self_hours',
  'first_review_hours',
  'final_review_hours',
  'approved_hours',
  'status',
  'rejection_reason',
  'recommended_hours',
  'custom.{字段Key}',
]

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

const loadImportTemplates = async () => {
  await importRequest.run(async () => {
    importTemplates.value = await listImportTemplates()
  })
}

const applySelectedImport = () => {
  const template = importTemplates.value.find((item) => item.template_key === selectedImportKey.value)
  if (!template) return
  importEdit.name = template.name
  importEdit.fields = template.fields.map((field) => ({ ...field }))
}

const handleSaveImportTemplate = async () => {
  await importSaveRequest.run(
    async () => {
      const payload = {
        name: importEdit.name,
        fields: importEdit.fields.map((field) => ({ ...field })),
      }
      const updated = await updateImportTemplate(selectedImportKey.value, payload)
      const index = importTemplates.value.findIndex(
        (item) => item.template_key === selectedImportKey.value,
      )
      if (index >= 0) {
        importTemplates.value[index] = updated
      }
      applySelectedImport()
    },
    { successMessage: '导入模板已保存' },
  )
}

const loadExportTemplate = async () => {
  await exportRequest.run(async () => {
  exportTemplate.value = await getExportTemplate('labor_hours')
  applyExportTemplate()
})
}

const applyExportTemplate = () => {
  if (!exportTemplate.value) return
  exportTemplateName.value = exportTemplate.value.name
  const layout = exportTemplate.value.layout as ExportLayout
  exportLayout.title = layout.title || ''
  exportLayout.sections = Array.isArray(layout.sections) ? layout.sections : []
  exportLayout.signature = layout.signature || {
    first_label: '初审教师签名',
    final_label: '复审教师签名',
  }
}

const handleSaveExportTemplate = async () => {
  await exportSaveRequest.run(
    async () => {
      const payload = {
        name: exportTemplateName.value || '劳动教育学时认定表',
        layout: {
          title: exportLayout.title,
          sections: exportLayout.sections,
          signature: exportLayout.signature,
        },
      }
      exportTemplate.value = await updateExportTemplate('labor_hours', payload)
      exportTemplateName.value = exportTemplate.value.name
    },
    { successMessage: '导出模板已保存' },
  )
}

const addInfoSection = () => {
  exportLayout.sections.push({ type: 'info', title: '信息区块', fields: [] })
}

const addTableSection = () => {
  exportLayout.sections.push({ type: 'table', title: '表格区块', columns: [] })
}

const addSectionField = (section: LayoutSection) => {
  if (section.type === 'info') {
    section.fields.push({ key: '', label: '' })
  } else {
    section.columns.push({ key: '', label: '' })
  }
}

const removeSectionField = (section: LayoutSection, index: number) => {
  if (section.type === 'info') {
    section.fields.splice(index, 1)
  } else {
    section.columns.splice(index, 1)
  }
}

const removeSection = (index: number) => {
  exportLayout.sections.splice(index, 1)
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

watch(selectedImportKey, applySelectedImport)

onMounted(() => {
  void loadFormFields()
  void loadImportTemplates().then(applySelectedImport)
  void loadExportTemplate()
  void loadLaborRules()
})
</script>

<template>
  <section class="hero">
    <h1>模板与规则配置</h1>
    <p>维护表单字段、导入映射、导出模板与劳动学时规则。</p>
  </section>

  <el-tabs v-model="activeTab" type="card">
    <el-tab-pane label="表单字段" name="fields">
      <div class="card-grid">
        <el-card class="card">
          <h3>新增字段</h3>
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

    <el-tab-pane label="导入模板" name="import">
      <el-card class="card">
        <div class="form-row">
          <el-select v-model="selectedImportKey" style="min-width: 240px">
            <el-option v-for="opt in importOptions" :key="opt.value" :label="opt.label" :value="opt.value" />
          </el-select>
          <el-input v-model="importEdit.name" placeholder="模板名称" style="max-width: 260px" />
          <el-button type="primary" :loading="importSaveRequest.loading" @click="handleSaveImportTemplate">
            保存模板
          </el-button>
        </div>

        <el-table :data="importEdit.fields" style="margin-top: 16px">
          <el-table-column label="排序" width="80">
            <template #default="{ row }">
              <el-input-number v-model="row.order_index" :min="1" />
            </template>
          </el-table-column>
          <el-table-column label="字段 Key" width="160">
            <template #default="{ row }">
              <el-input v-model="row.field_key" disabled />
            </template>
          </el-table-column>
          <el-table-column label="字段标签" width="160">
            <template #default="{ row }">
              <el-input v-model="row.label" />
            </template>
          </el-table-column>
          <el-table-column label="表头映射">
            <template #default="{ row }">
              <el-input v-model="row.column_title" />
            </template>
          </el-table-column>
          <el-table-column label="必填" width="90">
            <template #default="{ row }">
              <el-switch v-model="row.required" />
            </template>
          </el-table-column>
          <el-table-column label="说明">
            <template #default="{ row }">
              <el-input v-model="row.description" />
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </el-tab-pane>

    <el-tab-pane label="导出模板" name="export">
      <el-card class="card">
        <div class="form-row">
          <el-input v-model="exportTemplateName" placeholder="模板名称" style="max-width: 260px" />
          <el-input v-model="exportLayout.title" placeholder="PDF 标题" style="max-width: 260px" />
          <el-button type="primary" :loading="exportSaveRequest.loading" @click="handleSaveExportTemplate">
            保存导出模板
          </el-button>
        </div>
        <div class="hint" style="margin-top: 12px">
          <strong>可用字段 Key：</strong>
          <span>{{ templateKeysHint.join('、') }}</span>
        </div>

        <div class="section-actions" style="margin-top: 16px">
          <el-button @click="addInfoSection">新增信息区块</el-button>
          <el-button @click="addTableSection">新增表格区块</el-button>
        </div>

        <div class="card-grid" style="margin-top: 16px">
          <el-card v-for="(section, index) in exportLayout.sections" :key="index" class="card">
            <div class="form-row">
              <el-tag type="info">{{ section.type === 'info' ? '信息区块' : '表格区块' }}</el-tag>
              <el-input v-model="section.title" placeholder="区块标题" />
              <el-button type="danger" plain @click="removeSection(index)">删除区块</el-button>
            </div>
            <el-table
              :data="section.type === 'info' ? section.fields : section.columns"
              style="margin-top: 12px"
            >
              <el-table-column label="字段 Key">
                <template #default="{ row }">
                  <el-input v-model="row.key" placeholder="student_no / contest_name / custom.xxx" />
                </template>
              </el-table-column>
              <el-table-column label="字段标签">
                <template #default="{ row }">
                  <el-input v-model="row.label" placeholder="显示名称" />
                </template>
              </el-table-column>
              <el-table-column label="操作" width="120">
                <template #default="{ $index }">
                  <el-button type="danger" text @click="removeSectionField(section, $index)">
                    删除
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
            <el-button style="margin-top: 8px" @click="addSectionField(section)">新增字段</el-button>
          </el-card>
        </div>

        <el-divider />
        <h3>签名配置</h3>
        <el-form label-position="top" style="max-width: 320px">
          <el-form-item label="初审签名标签">
            <el-input v-model="exportLayout.signature!.first_label" />
          </el-form-item>
          <el-form-item label="复审签名标签">
            <el-input v-model="exportLayout.signature!.final_label" />
          </el-form-item>
        </el-form>
      </el-card>
    </el-tab-pane>

    <el-tab-pane label="学时规则" name="rules">
      <el-card class="card">
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
      importRequest.error ||
      importSaveRequest.error ||
      exportRequest.error ||
      exportSaveRequest.error ||
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
      importRequest.error ||
      importSaveRequest.error ||
      exportRequest.error ||
      exportSaveRequest.error ||
      laborRequest.error ||
      laborSaveRequest.error
    "
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
