<template>
  <div class="adt-root">
    <div class="adt-toolbar" v-if="$slots.toolbar || $slots['search-bar']">
      <div class="adt-search" v-if="$slots['search-bar']">
        <slot name="search-bar" />
      </div>
      <div class="adt-actions" v-if="$slots.toolbar">
        <slot name="toolbar" />
      </div>
    </div>
    <el-table
      ref="tableRef"
      v-bind="$attrs"
      :data="data"
      :border="border"
      stripe
      v-loading="loading"
      :empty-text="emptyText"
      size="default"
      @selection-change="$emit('selection-change', $event)"
    >
      <slot />
    </el-table>
    <div class="adt-pager" v-if="showPagination && total > pageSize">
      <el-pagination
        :current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="prev, pager, next, total"
        @current-change="$emit('update:page', $event)"
      />
    </div>
  </div>
</template>

<script>
export default {
  name: 'AdminDataTable',
  inheritAttrs: false,
  props: {
    data: { type: Array, default: () => [] },
    loading: { type: Boolean, default: false },
    page: { type: Number, default: 1 },
    pageSize: { type: Number, default: 20 },
    total: { type: Number, default: 0 },
    showPagination: { type: Boolean, default: true },
    border: { type: Boolean, default: false },
    emptyText: { type: String, default: '暂无数据' }
  },
  emits: ['update:page', 'selection-change'],
  methods: {
    getTableRef() { return this.$refs.tableRef; }
  }
};
</script>

<style scoped>
.adt-root { }
.adt-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 14px;
}
.adt-search {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.adt-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.adt-pager {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
