<script setup lang="ts"></script>

<template>
  <section class="hero">
    <h1>设备与恢复码</h1>
    <p>管理 Passkey/TOTP 设备，生成恢复码。</p>
  </section>

  <div class="card-grid">
    <article class="card">
      <h3>已绑定设备</h3>
      <p>当前设备列表为空。</p>
      <button class="button secondary">刷新列表</button>
    </article>
    <article class="card">
      <h3>恢复码</h3>
      <p>请妥善保存最新恢复码。</p>
      <button class="button">生成恢复码</button>
    </article>
  </div>
</template>
