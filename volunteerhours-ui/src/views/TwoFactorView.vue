<script setup lang="ts"></script>

<template>
  <section class="hero">
    <h1>二次验证</h1>
    <p>输入 TOTP 动态码或恢复码以完成登录。</p>
  </section>

  <div class="card">
    <div class="form-field">
      <label>验证码</label>
      <input placeholder="6 位动态码或恢复码" />
    </div>
    <div class="form-field">
      <label>设备标记</label>
      <input placeholder="这台设备的名称" />
    </div>
    <button class="button">验证并进入</button>
  </div>
</template>
