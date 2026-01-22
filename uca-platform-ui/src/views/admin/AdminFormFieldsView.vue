<script setup lang="ts">
import { reactive, ref } from 'vue'
import { createFormField, listFormFields } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const formFieldRef = ref()
const formFields = ref('')
const result = ref('')
const formFieldRequest = useRequest()
const listRequest = useRequest()

const formField = reactive({
  form_type: 'volunteer',
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
</script>

<template>
  <section class="hero">
    <h1>模板配置</h1>
    <p>维护志愿服务与竞赛表单字段。</p>
  </section>

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
    v-if="formFieldRequest.error || listRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="formFieldRequest.error || listRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
