<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const message = ref("");
const saving = ref(false);

watch(visible, (v) => {
  if (v) message.value = "";
});

async function confirmStash() {
  saving.value = true;
  try {
    await props.git.stashSave(message.value);
    visible.value = false;
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Cất thay đổi vào stash" :style="{ width: '460px' }">
    <div class="flex flex-col gap-3">
      <p class="text-sm text-secondary">
        Toàn bộ thay đổi chưa commit ({{ git.staged.value.length + git.unstaged.value.length }} file) sẽ được
        cất vào stash và working tree sẽ trở về sạch. Bạn có thể áp dụng lại sau trong "Quản lý stash…".
      </p>
      <div>
        <label class="mb-1 block text-sm font-medium text-ink">Message (tuỳ chọn)</label>
        <InputText
          v-model="message"
          placeholder="Mô tả ngắn cho lần stash này…"
          class="w-full"
          @keydown.enter="confirmStash"
        />
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Hủy</Button>
      <Button size="small" :loading="saving" @click="confirmStash">
        <i class="pi pi-inbox mr-1.5" /> Cất vào stash
      </Button>
    </template>
  </Dialog>
</template>
