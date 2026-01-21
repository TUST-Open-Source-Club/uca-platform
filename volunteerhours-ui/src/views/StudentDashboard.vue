<script setup lang="ts">
import { reactive, ref } from 'vue'
import { createContest, createVolunteer } from '../api/records'
import { useRequest } from '../composables/useRequest'

const volunteerFormRef = ref()
const contestFormRef = ref()
const result = ref('')
const volunteerRequest = useRequest()
const contestRequest = useRequest()

const volunteerForm = reactive({
  title: '',
  self_hours: 0,
  description: '',
})

const contestForm = reactive({
  contest_name: '',
  award_level: '',
  self_hours: 0,
})

const validateHours = (_: unknown, value: number, callback: (error?: Error) => void) => {
  if (Number(value) <= 0) {
    callback(new Error('学时需大于 0'))
    return
  }
  callback()
}

const volunteerRules = {
  title: [{ required: true, message: '请输入标题', trigger: 'blur' }],
  self_hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
}

const contestRules = {
  contest_name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
  award_level: [{ required: true, message: '请输入获奖等级', trigger: 'blur' }],
  self_hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
}

const handleVolunteerSubmit = async () => {
  if (!volunteerFormRef.value) return
  await volunteerFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await volunteerRequest.run(
      async () => {
        const data = await createVolunteer({
          title: volunteerForm.title,
          description: volunteerForm.description,
          self_hours: Number(volunteerForm.self_hours),
        })
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已提交志愿服务' },
    )
  })
}

const handleContestSubmit = async () => {
  if (!contestFormRef.value) return
  await contestFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await contestRequest.run(
      async () => {
        const data = await createContest({
          contest_name: contestForm.contest_name,
          award_level: contestForm.award_level,
          self_hours: Number(contestForm.self_hours),
        })
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已提交竞赛获奖' },
    )
  })
}
</script>

<template>
  <section class="hero">
    <h1>学生中心</h1>
    <p>提交志愿服务与竞赛获奖记录，跟踪审核进度。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>志愿服务填报</h3>
      <el-form ref="volunteerFormRef" :model="volunteerForm" :rules="volunteerRules" label-position="top">
        <el-form-item label="标题" prop="title">
          <el-input v-model="volunteerForm.title" placeholder="活动名称" />
        </el-form-item>
        <el-form-item label="自评学时" prop="self_hours">
          <el-input-number v-model="volunteerForm.self_hours" :min="0" />
        </el-form-item>
        <el-form-item label="说明">
          <el-input v-model="volunteerForm.description" type="textarea" rows="3" />
        </el-form-item>
        <el-button type="primary" :loading="volunteerRequest.loading" @click="handleVolunteerSubmit">
          提交
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛获奖填报</h3>
      <el-form ref="contestFormRef" :model="contestForm" :rules="contestRules" label-position="top">
        <el-form-item label="竞赛名称" prop="contest_name">
          <el-input v-model="contestForm.contest_name" placeholder="竞赛全称" />
        </el-form-item>
        <el-form-item label="获奖等级" prop="award_level">
          <el-input v-model="contestForm.award_level" placeholder="例如 省赛一等奖" />
        </el-form-item>
        <el-form-item label="自评学时" prop="self_hours">
          <el-input-number v-model="contestForm.self_hours" :min="0" />
        </el-form-item>
        <el-button type="primary" :loading="contestRequest.loading" @click="handleContestSubmit">
          提交
        </el-button>
      </el-form>
    </el-card>
  </div>

  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>

  <el-alert
    v-if="volunteerRequest.error || contestRequest.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="volunteerRequest.error || contestRequest.error"
    :closable="false"
  />
</template>
