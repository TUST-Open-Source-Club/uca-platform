<script setup lang="ts"></script>

<template>
  <section class="hero">
    <h1>管理台</h1>
    <p>维护学生名单、竞赛名称库与模板配置。</p>
  </section>

  <div class="card-grid">
    <article class="card">
      <h3>学生名单导入</h3>
      <p>支持腾讯文档导出 Excel。</p>
      <button class="button">上传名单</button>
    </article>
    <article class="card">
      <h3>竞赛名称库</h3>
      <p>维护标准竞赛名称，便于自动比对。</p>
      <button class="button secondary">管理竞赛库</button>
    </article>
    <article class="card">
      <h3>模板配置</h3>
      <p>导入/导出字段自定义布局。</p>
      <button class="button secondary">配置模板</button>
    </article>
  </div>
</template>
