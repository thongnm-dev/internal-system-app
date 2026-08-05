<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Dialog from "primevue/dialog";
import Select from "primevue/select";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";
import { guessBase } from "../utils/gitRefs";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const updateBranchSel = ref("");

const allBranchRefs = computed(() =>
  props.git.branches.value
    .filter((b) => !b.name.endsWith("/HEAD"))
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);

watch(visible, (v) => {
  if (v) {
    updateBranchSel.value = guessBase(
      props.git.branches.value.map((b) => b.name),
      props.git.info.value?.current_branch || "",
      props.git.info.value?.upstream,
    );
  }
});

async function doUpdateFromMain() {
  if (!updateBranchSel.value) return;
  const ok = await props.git.mergeBranch(updateBranchSel.value, false, "");
  if (ok) visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Cập nhật từ main/master" :style="{ width: '460px' }">
    <div class="flex flex-col gap-3">
      <p class="text-sm text-secondary">
        Merge nhánh mặc định vào <strong class="text-ink">{{ git.info.value?.current_branch }}</strong>:
      </p>
      <Select
        v-model="updateBranchSel"
        :options="allBranchRefs"
        option-label="label"
        option-value="value"
        filter
        class="w-full"
      />
      <p class="text-xs text-muted">
        Nếu có xung đột, dùng nút "Xử lý xung đột" trên thanh cảnh báo để giải quyết.
      </p>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Cập nhật"
        confirm-icon="pi pi-arrow-circle-down"
        :confirm-disabled="!updateBranchSel || !!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doUpdateFromMain"
      />
    </template>
  </Dialog>
</template>
