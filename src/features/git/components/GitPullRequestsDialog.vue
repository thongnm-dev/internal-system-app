<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });
const emit = defineEmits<{
  "open-compare": [pr: { base: string; head: string } | null];
}>();

const prStateFilter = ref<"open" | "closed" | "all">("open");
const isMaximized = ref(false);

watch(visible, (v) => {
  if (v) {
    prStateFilter.value = "open";
    isMaximized.value = false;
    props.git.loadPullRequests("open");
  }
});

function changePrState(state: "open" | "closed" | "all") {
  prStateFilter.value = state;
  props.git.loadPullRequests(state);
}

function prStateBadge(s: string) {
  if (s === "merged") return "badge-info";
  if (s === "closed") return "badge-danger";
  if (s === "draft") return "badge-neutral";
  return "badge-success";
}

function viewPrDiff(pr: { base: string; head: string }) {
  visible.value = false;
  emit("open-compare", pr);
}

function createNewPr() {
  visible.value = false;
  emit("open-compare", null);
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    maximizable
    header="Pull Requests"
    :style="{ width: '720px' }"
    @maximize="isMaximized = true"
    @unmaximize="isMaximized = false"
  >
    <div class="flex flex-col gap-3">
      <div class="flex items-center gap-2">
        <div class="flex overflow-hidden rounded-md border border-divider">
          <button
            v-for="opt in (['open','closed','all'] as const)"
            :key="opt"
            class="px-3 py-1 text-xs font-medium transition-colors"
            :class="prStateFilter === opt ? 'bg-brand text-white' : 'text-secondary hover:bg-canvas'"
            @click="changePrState(opt)"
          >
            {{ opt === 'open' ? 'Đang mở' : opt === 'closed' ? 'Đã đóng' : 'Tất cả' }}
          </button>
        </div>
        <button
          class="rounded p-1.5 text-muted transition-colors hover:bg-canvas hover:text-brand"
          title="Làm mới"
          @click="git.loadPullRequests(prStateFilter)"
        >
          <i class="pi text-xs" :class="git.pullRequestsLoading.value ? 'pi-spinner pi-spin' : 'pi-refresh'" />
        </button>
        <span class="ml-auto text-xs text-muted">Dùng credential git đã lưu để truy cập repo riêng tư.</span>
      </div>

      <div
        class="overflow-y-auto rounded-md border border-divider"
        :class="isMaximized ? 'h-[calc(100vh-260px)]' : 'min-h-[280px] max-h-[440px]'"
      >
        <div v-if="git.pullRequestsLoading.value" class="p-8 text-center text-sm text-muted">
          <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải…
        </div>
        <div v-else-if="!git.pullRequests.value.length" class="p-8 text-center text-sm text-muted">
          Không có Pull Request nào.
        </div>
        <div
          v-for="pr in git.pullRequests.value"
          v-else
          :key="pr.number"
          class="group flex items-start gap-3 border-b border-divider-light px-3 py-2.5 transition-colors last:border-0 hover:bg-canvas"
        >
          <span class="mt-0.5 shrink-0 uppercase" :class="prStateBadge(pr.state)">
            {{ pr.state }}
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-sm font-medium text-ink">
              #{{ pr.number }} {{ pr.title }}
            </span>
            <span class="mt-0.5 flex flex-wrap items-center gap-x-2 text-[11px] text-muted">
              <span>{{ pr.author }}</span>
              <span class="font-mono">{{ pr.head }} → {{ pr.base }}</span>
            </span>
          </span>
          <button
            class="shrink-0 rounded p-1 text-muted transition-colors hover:bg-panel hover:text-brand"
            title="Xem diff của PR"
            @click="viewPrDiff(pr)"
          >
            <i class="pi pi-file-edit text-xs" />
          </button>
          <button
            class="shrink-0 rounded p-1 text-muted transition-colors hover:bg-panel hover:text-brand"
            title="Mở trên trình duyệt"
            @click="git.openUrl(pr.url)"
          >
            <i class="pi pi-external-link text-xs" />
          </button>
        </div>
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
      <Button size="small" @click="createNewPr">
        <i class="pi pi-plus mr-1.5" /> Tạo Pull Request
      </Button>
    </template>
  </Dialog>
</template>
