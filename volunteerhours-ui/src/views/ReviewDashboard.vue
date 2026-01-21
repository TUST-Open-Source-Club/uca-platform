<script setup lang="ts"></script>

<template>
  <section class="hero">
    <h1>审核中心</h1>
    <p>初审与复审队列集中管理。</p>
  </section>

  <div class="card-grid">
    <article class="card">
      <h3>待初审</h3>
      <p>0 条记录</p>
      <button class="button">进入初审</button>
    </article>
    <article class="card">
      <h3>待复审</h3>
      <p>0 条记录</p>
      <button class="button">进入复审</button>
    </article>
    <article class="card">
      <h3>签名与意见</h3>
      <p>上传电子签名并填写审核意见。</p>
      <button class="button secondary">管理签名</button>
    </article>
  </div>
</template>
