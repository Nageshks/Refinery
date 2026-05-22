<script setup lang="ts">
interface Props {
  size?: number | string;
}

withDefaults(defineProps<Props>(), {
  size: 20,
});
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="refinery-brand-icon"
  >
    <defs>
      <!-- Background sheet gradient -->
      <linearGradient id="bg-sheet-grad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="#FFFFFF" />
        <stop offset="100%" stop-color="#C7CAD0" />
      </linearGradient>
      
      <!-- Glass sheet gradient -->
      <linearGradient id="glass-sheet-grad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="rgba(167, 139, 250, 0.6)" />
        <stop offset="100%" stop-color="rgba(109, 40, 217, 0.35)" />
      </linearGradient>
      
      <!-- Drop shadow for the glass overlay -->
      <filter id="glass-shadow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow dx="0" dy="1.5" stdDeviation="1.2" flood-color="rgba(0, 0, 0, 0.4)" flood-opacity="0.25" />
      </filter>
    </defs>

    <!-- Background Document (Original Draft) -->
    <path 
      d="M 3.5 2 L 11 2 L 16 7 L 16 19.5 C 16 20.3 15.3 21 14.5 21 L 3.5 21 C 2.7 21 2 20.3 2 19.5 L 2 3.5 C 2 2.7 2.7 2 3.5 2 Z" 
      fill="url(#bg-sheet-grad)"
    />
    <!-- Document Fold Corner -->
    <path 
      d="M 11 2 L 11 5.5 C 11 6.3 11.7 7 12.5 7 L 16 7 Z" 
      fill="#9CA3AF" 
    />
    
    <!-- Background Doc Lines -->
    <rect x="4.5" y="10" width="8" height="1.5" rx="0.75" fill="#9CA3AF" />
    <rect x="4.5" y="13" width="8" height="1.5" rx="0.75" fill="#9CA3AF" />
    <rect x="4.5" y="16" width="5" height="1.5" rx="0.75" fill="#9CA3AF" />

    <!-- Foreground Glass Document Overlay (Refined Draft) -->
    <rect 
      x="9" 
      y="9" 
      width="13" 
      height="13" 
      rx="2.5" 
      fill="url(#glass-sheet-grad)" 
      stroke="rgba(167, 139, 250, 0.9)" 
      stroke-width="0.8" 
      filter="url(#glass-shadow)"
    />
    
    <!-- Glass Doc Lines -->
    <rect x="11.5" y="13" width="8" height="1.5" rx="0.75" fill="#FFFFFF" />
    <rect x="11.5" y="16" width="5" height="1.5" rx="0.75" fill="#FFFFFF" />
  </svg>
</template>

<style scoped>
.refinery-brand-icon {
  display: inline-block;
  vertical-align: middle;
  transition: transform var(--transition-fast);
}

.refinery-brand-icon:hover {
  transform: translateY(-1px) scale(1.06);
}
</style>
