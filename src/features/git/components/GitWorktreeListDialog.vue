<script setup lang="ts">
import { watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });
const emit = defineEmits<{ "create-worktree": [] }>();

watch(visible, (v) => {
  if (v) props.git.loadWorktrees();
});

function requestCreate() {
  visible.value = false;
  emit("create-worktree");
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Worktrees" :style="{ width: '620px' }">
    <div class="flex flex-col gap-2">
      <div
        v-for="w in git.worktrees.value"
        :key="w.path"
        class="flex items-center gap-2 rounded-md border border-divider px-3 py-2"
      >
        <i class="pi shrink-0 text-sm" :class="w.is_current ? 'pi-check-circle text-brand' : 'pi-folder text-muted'" />
        <div class="min-w-0 flex-1">
          <p class="truncate font-mono text-xs text-ink">{{ w.path }}</p>
          <p class="text-[11px] text-muted">
            <span v-if="w.is_bare">bare</span>
            <span v-else-if="w.is_detached">detached @ {{ w.head.slice(0, 7) }}</span>
            <span v-else>{{ w.branch }}</span>
            <span v-if="w.is_current" class="ml-1 text-brand">· đang mở</span>
          </p>
        </div>
        <Button
          v-if="!w.is_current"
          size="small"
          outlined
          severity="secondary"
          class="shrink-0"
          title="Mở worktree này"
          @click="visible = false; git.openPathAsRepo(w.path)"
        >
          <i class="pi pi-external-link" />
        </Button>
        <Button
          v-if="!w.is_current && !w.is_bare"
          size="small"
          outlined
          severity="danger"
          class="shrink-0"
          title="Gỡ worktree"
          @click="git.worktreeRemove(w.path, false)"
        >
          <i class="pi pi-trash" />
        </Button>
      </div>
      <div v-if="!git.worktrees.value.length" class="p-4 text-center text-sm text-muted">
        Chưa có worktree nào.
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
      <Button size="small" @click="requestCreate">
        <i class="pi pi-plus mr-1.5" /> Tạo worktree
      </Button>
    </template>
  </Dialog>
</template>
