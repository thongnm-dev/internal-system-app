<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Dialog from "primevue/dialog";
import Select from "primevue/select";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const rebaseTarget = ref("");

const rebaseOptions = computed(() =>
  props.git.branches.value
    .filter((b) => !b.is_current)
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);

watch(visible, (v) => {
  if (v) rebaseTarget.value = "";
});

async function doRebase() {
  if (!rebaseTarget.value) return;
  await props.git.rebaseOnto(rebaseTarget.value);
  visible.value = false;
  rebaseTarget.value = "";
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Rebase branch hiện tại" :style="{ width: '460px' }">
    <div class="flex flex-col gap-3">
      <p class="text-sm text-secondary">
        Rebase <strong class="text-ink">{{ git.info.value?.current_branch }}</strong> lên trên branch:
      </p>
      <Select
        v-model="rebaseTarget"
        :options="rebaseOptions"
        option-label="label"
        option-value="value"
        placeholder="Chọn branch đích…"
        filter
        class="w-full"
      />
      <p class="text-xs text-muted">
        Nếu có xung đột, rebase sẽ tạm dừng — bạn giải quyết ở tab Changes rồi bấm "Tiếp tục" trên thanh cảnh báo.
      </p>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Rebase"
        confirm-icon="pi pi-arrows-v"
        :confirm-disabled="!rebaseTarget || !!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doRebase"
      />
    </template>
  </Dialog>
</template>
