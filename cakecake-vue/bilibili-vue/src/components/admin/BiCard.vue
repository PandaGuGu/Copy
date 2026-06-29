<template>
  <div class="bi-card" :class="[`bi-card--${color}`]">
    <div class="bi-card__icon" v-if="icon">{{ icon }}</div>
    <div class="bi-card__body">
      <span class="bi-card__label">{{ label }}</span>
      <span class="bi-card__value">{{ value }}</span>
      <span v-if="trend !== undefined" class="bi-card__trend" :class="{ 'is-up': trend > 0, 'is-down': trend < 0 }">
        {{ trend > 0 ? '+' : '' }}{{ trend }}%
      </span>
    </div>
    <div v-if="sub" class="bi-card__sub">{{ sub }}</div>
  </div>
</template>

<script setup>
defineProps({
  label: { type: String, default: '' },
  value: { type: [String, Number], default: '' },
  trend: { type: Number, default: undefined },
  sub: { type: String, default: '' },
  icon: { type: String, default: '' },
  color: { type: String, default: 'blue' }
})
</script>

<style scoped>
.bi-card {
  background: #fff;
  border: 1px solid #e3e5e7;
  border-radius: 10px;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: box-shadow 0.2s, transform 0.2s;
  cursor: default;
}
.bi-card:hover {
  box-shadow: 0 2px 12px rgba(0,0,0,0.06);
  transform: translateY(-1px);
}
.bi-card__body {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: wrap;
}
.bi-card__label {
  font-size: 13px;
  color: #9499a0;
}
.bi-card__value {
  font-size: 26px;
  font-weight: 600;
  color: #18191c;
  line-height: 1.2;
}
.bi-card__trend {
  font-size: 12px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: 4px;
}
.bi-card__trend.is-up {
  color: #02b340;
  background: rgba(2, 179, 64, 0.08);
}
.bi-card__trend.is-down {
  color: #e24b4a;
  background: rgba(226, 75, 74, 0.08);
}
.bi-card__sub {
  font-size: 12px;
  color: #c9ccd0;
}

/* Color accent border-left */
.bi-card--blue { border-left: 3px solid #378add; }
.bi-card--teal { border-left: 3px solid #1d9e75; }
.bi-card--purple { border-left: 3px solid #7f77dd; }
.bi-card--amber { border-left: 3px solid #ba7517; }
.bi-card--pink { border-left: 3px solid #d4537e; }
.bi-card--coral { border-left: 3px solid #d85a30; }
</style>
