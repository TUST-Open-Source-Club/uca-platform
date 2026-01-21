<script setup lang="ts">
const methods = [
  { id: 'passkey', title: 'Passkey 登录', desc: '使用设备生物识别或安全密钥' },
  { id: 'totp', title: 'TOTP 登录', desc: '输入动态验证码' },
]
</script>

<template>
  <section class="hero">
    <h1>欢迎进入志愿时长与竞赛统计平台</h1>
    <p>请使用 Passkey 或 TOTP 完成认证。系统不支持纯密码登录。</p>
  </section>

  <div class="card-grid">
    <article v-for="method in methods" :key="method.id" class="card">
      <h3>{{ method.title }}</h3>
      <p>{{ method.desc }}</p>
      <button class="button">开始</button>
    </article>
  </div>

  <div class="card" style="margin-top: 24px">
    <h3>账户输入</h3>
    <div class="form-field">
      <label>学号 / 工号</label>
      <input placeholder="请输入学号或工号" />
    </div>
    <div class="form-field">
      <label>认证方式</label>
      <select>
        <option>Passkey</option>
        <option>TOTP</option>
        <option>恢复码</option>
      </select>
    </div>
    <button class="button">进入认证</button>
  </div>
</template>
