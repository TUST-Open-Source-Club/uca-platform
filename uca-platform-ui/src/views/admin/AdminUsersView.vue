<script setup lang="ts">
import { reactive, ref } from 'vue'
import { createUser } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const userFormRef = ref()
const result = ref('')
const userRequest = useRequest()

const userForm = reactive({
  username: '',
  display_name: '',
  role: 'student',
  email: '',
})

const userRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  display_name: [{ required: true, message: '请输入显示名称', trigger: 'blur' }],
  role: [{ required: true, message: '请选择角色', trigger: 'change' }],
  email: [
    {
      validator: (_rule: unknown, value: string, callback: (err?: Error) => void) => {
        if (userForm.role !== 'student' && !value) {
          callback(new Error('非学生必须填写邮箱'))
          return
        }
        callback()
      },
      trigger: 'blur',
    },
  ],
}

const handleCreateUser = async () => {
  if (!userFormRef.value) return
  await userFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await userRequest.run(async () => {
      const payload = {
        username: userForm.username,
        display_name: userForm.display_name,
        role: userForm.role as 'student' | 'teacher' | 'reviewer' | 'admin',
        email: userForm.email || undefined,
      }
      const data = await createUser(payload)
      result.value = JSON.stringify(data, null, 2)
    }, { successMessage: '已提交用户创建' })
  })
}
</script>

<template>
  <section class="hero">
    <h1>创建用户 / 发送邀请</h1>
    <p>新增用户并发送邀请邮件。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <el-form ref="userFormRef" :model="userForm" :rules="userRules" label-position="top">
        <el-form-item label="用户名" prop="username">
          <el-input v-model="userForm.username" placeholder="学号或工号" />
        </el-form-item>
        <el-form-item label="显示名称" prop="display_name">
          <el-input v-model="userForm.display_name" placeholder="姓名" />
        </el-form-item>
        <el-form-item label="角色" prop="role">
          <el-select v-model="userForm.role">
            <el-option label="学生" value="student" />
            <el-option label="教师" value="teacher" />
            <el-option label="审核员" value="reviewer" />
            <el-option label="管理员" value="admin" />
          </el-select>
        </el-form-item>
        <el-form-item label="邮箱（非学生必填）" prop="email">
          <el-input v-model="userForm.email" placeholder="user@example.com" />
        </el-form-item>
        <el-button type="primary" :loading="userRequest.loading" @click="handleCreateUser">
          提交
        </el-button>
      </el-form>
    </el-card>
  </div>

  <el-alert
    v-if="userRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="userRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
