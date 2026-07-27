<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const resetTarget = ref("HEAD");
const resetMode = ref<"soft" | "mixed" | "hard">("mixed");

watch(visible, (v) => {
  if (v) {
    resetTarget.value = "HEAD";
    resetMode.value = "mixed";
  }
});

async function doResetHead() {
  await props.git.resetTo(resetTarget.value.trim() || "HEAD", resetMode.value);
  visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Reset HEAD" :style="{ width: '460px' }">
    <div class="flex flex-col gap-3">
      <div>
        <label class="mb-1 block text-sm font-medium text-ink">Reset về (ref/commit)</label>
        <InputText v-model="resetTarget" placeholder="HEAD, HEAD~1, origin/main…" class="w-full" />
      </div>
      <div>
        <label class="mb-1 block text-sm font-medium text-ink">Chế độ</label>
        <div class="flex overflow-hidden rounded-md border border-divider text-xs">
          <button
            v-for="m in (['soft','mixed','hard'] as const)"
            :key="m"
            class="flex-1 px-2 py-1.5 font-medium transition-colors"
            :class="resetMode === m ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
            @click="resetMode = m"
          >
            {{ m }}
          </button>
        </div>
        <p class="mt-1 text-xs text-muted">
          <template v-if="resetMode === 'soft'">Giữ nguyên index và working tree (chỉ dời HEAD).</template>
          <template v-else-if="resetMode === 'mixed'">Giữ working tree, bỏ stage (mặc định).</template>
          <template v-else class="text-red-600">Xóa toàn bộ thay đổi working tree — không hoàn tác được.</template>
        </p>
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Hủy</Button>
      <Button
        size="small"
        :severity="resetMode === 'hard' ? 'danger' : undefined"
        :disabled="!!git.busyMessage.value"
        @click="doResetHead"
      >
        <i class="pi pi-backward mr-1.5" /> Reset ({{ resetMode }})
      </Button>
    </template>
  </Dialog>
</template>
