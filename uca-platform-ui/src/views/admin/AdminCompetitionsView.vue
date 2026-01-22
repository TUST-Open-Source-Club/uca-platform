<script setup lang="ts">
import { reactive, ref } from 'vue'
import { createCompetition, listCompetitions } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const competitionFormRef = ref()
const competitions = ref('')
const result = ref('')
const competitionRequest = useRequest()
const listRequest = useRequest()

const competitionForm = reactive({
  name: '',
  year: 0,
  category: '',
})

const competitionRules = {
  name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
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
        const data = await createCompetition({
          name: competitionForm.name,
          year: competitionForm.year || null,
          category: competitionForm.category || null,
        })
        result.value = JSON.stringify(data, null, 2)
        await loadCompetitions()
      },
      { successMessage: '已新增竞赛' },
    )
  })
}
</script>

<template>
  <section class="hero">
    <h1>竞赛库管理</h1>
    <p>新增竞赛名称并查看当前列表。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>新增竞赛</h3>
        <el-form ref="competitionFormRef" :model="competitionForm" :rules="competitionRules" label-position="top">
          <el-form-item label="竞赛名称" prop="name">
            <el-input v-model="competitionForm.name" placeholder="竞赛全称" />
          </el-form-item>
          <el-form-item label="竞赛年份">
            <el-input-number v-model="competitionForm.year" :min="2000" :max="2100" />
          </el-form-item>
          <el-form-item label="竞赛类型">
            <el-select v-model="competitionForm.category" placeholder="A/B">
              <el-option label="A 类" value="A" />
              <el-option label="B 类" value="B" />
            </el-select>
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
  </div>

  <el-alert
    v-if="competitionRequest.error || listRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="competitionRequest.error || listRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
