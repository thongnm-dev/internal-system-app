<script setup lang="ts">
import { computed, ref } from "vue";
import Dialog from "primevue/dialog";
import Calendar from "primevue/calendar";
import { useAuthStore } from "@/app/stores/auth";
import { useDailyWorkNotes } from "../composables/useDailyWorkNotes";
import type { DailyWorkNoteDraft, DailyWorkStatus } from "../composables/useDailyWorkNotes";

const auth = useAuthStore();
const ctrl = useDailyWorkNotes(auth.user?.username);

const statusOptions: { icon: string; label: string; value: DailyWorkStatus }[] = [
  { icon: "pi-check-circle", label: "Hoàn thành", value: "completed" },
  { icon: "pi-clock", label: "Chưa hoàn thành", value: "incomplete" },
  { icon: "pi-circle", label: "Bảo lưu", value: "reserved" },
];

const weekdays = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];

const dialogVisible = ref(false);
const editingNoteId = ref<number | null>(null);
const draftContent = ref("");
const draftDate = ref<string>(ctrl.selectedDate.value);
const draftStatus = ref<DailyWorkStatus>("completed");

const isEditing = computed(() => editingNoteId.value !== null);

function openAddDialog() {
  editingNoteId.value = null;
  draftContent.value = "";
  draftDate.value = ctrl.selectedDate.value;
  draftStatus.value = "completed";
  dialogVisible.value = true;
}

function openEditDialog(note: { id: number; content: string; date: string; status: DailyWorkStatus }) {
  editingNoteId.value = note.id;
  draftContent.value = note.content;
  draftDate.value = note.date;
  draftStatus.value = note.status;
  dialogVisible.value = true;
}

async function saveDraft() {
  if (isEditing.value) {
    if (await ctrl.updateNoteContent(editingNoteId.value!, draftContent.value)) {
      const noteId = editingNoteId.value!;
      const newStatus = draftStatus.value;
      const currentNote = ctrl.filteredNotes.value.find((n) => n.id === noteId);
      if (currentNote && currentNote.status !== newStatus) {
        await ctrl.updateNoteStatus(noteId, newStatus);
      }
      dialogVisible.value = false;
    }
  } else {
    const dateValue = draftDate.value ? parseDateStr(draftDate.value) : null;
    const draft: DailyWorkNoteDraft = {
      content: draftContent.value,
      date: dateValue,
      status: draftStatus.value,
    };
    if (await ctrl.addNote(draft)) {
      dialogVisible.value = false;
    }
  }
}

function parseDateStr(value: string): Date | null {
  const [y, m, d] = value.split("-").map(Number);
  if (!y || !m || !d) return null;
  return new Date(y, m - 1, d);
}

</script>

<template>
  <section class="grid min-h-0 flex-1 grid-cols-[minmax(360px,460px)_minmax(0,1fr)] gap-4 overflow-hidden">
    <!-- Calendar panel -->
    <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex h-[76px] shrink-0 items-center justify-between gap-3 border-b border-divider px-4">
        <div class="flex min-w-0 items-center gap-3">
          <span class="flex h-10 w-10 items-center justify-center rounded-md bg-emerald-50 text-brand">
            <i class="pi pi-calendar text-xl" />
          </span>
          <div class="min-w-0">
            <h3 class="truncate font-bold capitalize">{{ ctrl.monthLabel.value }}</h3>
            <p class="mt-1 truncate text-xs text-muted">
              Tổng {{ ctrl.totalSelectedDateNotes.value }} công việc trong ngày đã chọn
            </p>
          </div>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <button
            class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
            type="button"
            title="Tháng trước"
            @click="ctrl.previousMonth()"
          >
            <i class="pi pi-chevron-left" />
          </button>
          <input
            class="h-9 w-32 rounded-md border border-divider bg-panel px-3 text-center text-sm font-bold text-secondary outline-none hover:border-brand focus:border-brand focus:ring-2 focus:ring-emerald-100"
            type="month"
            :value="ctrl.monthValue.value"
            @change="ctrl.selectMonth(($event.target as HTMLInputElement).value)"
          />
          <button
            class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
            type="button"
            title="Tháng sau"
            @click="ctrl.nextMonth()"
          >
            <i class="pi pi-chevron-right" />
          </button>
        </div>
      </div>

      <div class="grid grid-cols-7 border-b border-divider text-center text-xs font-bold text-ink">
        <div v-for="wd in weekdays" :key="wd" class="px-2 py-3">{{ wd }}</div>
      </div>

      <div class="grid flex-1 grid-cols-7 grid-rows-6 bg-panel">
        <button
          v-for="day in ctrl.calendarDays.value"
          :key="day.date"
          :class="[
            'min-h-0 border-b border-r border-divider p-2 text-left outline-none transition focus:ring-2 focus:ring-inset focus:ring-emerald-100',
            day.isSelected
              ? 'bg-emerald-50'
              : day.isCurrentMonth
                ? 'bg-panel hover:bg-canvas'
                : 'bg-canvas text-muted',
            day.isFutureDisabled ? 'cursor-not-allowed opacity-50' : '',
          ]"
          type="button"
          :disabled="day.isFutureDisabled"
          @click="ctrl.selectDate(day.date)"
        >
          <span
            :class="[
              'flex h-7 w-7 items-center justify-center rounded-md text-sm font-extrabold tabular-nums',
              day.isToday ? 'bg-brand text-white' : day.isSelected ? 'text-brand' : 'text-secondary',
            ]"
          >
            {{ day.day }}
          </span>
          <span class="mt-3 flex items-center justify-between gap-1">
            <span class="text-[11px] font-semibold text-muted">Công việc</span>
            <strong class="rounded-md bg-canvas px-2 py-1 text-xs text-secondary">{{ day.taskCount }}</strong>
          </span>
        </button>
      </div>
    </section>

    <!-- Notes panel -->
    <section class="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] gap-4 overflow-hidden">
      <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <h3 class="truncate font-bold">Ghi chú công việc hằng ngày</h3>
            <p class="mt-1 truncate text-sm text-muted">{{ ctrl.selectedDateLabel.value }}</p>
          </div>
          <button
            class="flex h-10 shrink-0 items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
            type="button"
            @click="openAddDialog"
          >
            <i class="pi pi-plus" />
            Thêm công việc
          </button>
        </div>
      </section>

      <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
        <div class="shrink-0 border-b border-divider p-4">
          <div class="grid grid-cols-3 rounded-md border border-divider bg-canvas p-1">
            <button
              v-for="opt in statusOptions"
              :key="opt.value"
              :class="[
                'flex h-10 items-center justify-center gap-2 rounded-md text-sm font-bold transition',
                ctrl.statusFilter.value === opt.value
                  ? 'bg-panel text-ink shadow-sm'
                  : 'text-muted hover:text-secondary',
              ]"
              type="button"
              @click="ctrl.statusFilter.value = opt.value"
            >
              <i :class="`pi ${opt.icon}`" />
              <span class="truncate">{{ opt.label }}</span>
              <span class="rounded-md bg-panel/70 px-1.5 py-0.5 text-xs">{{ ctrl.statusCounts.value[opt.value] }}</span>
            </button>
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-auto p-4">
          <div
            v-if="ctrl.filteredNotes.value.length === 0"
            class="flex h-full min-h-48 items-center justify-center rounded-md border border-dashed border-divider bg-canvas px-4 text-center text-sm font-semibold text-muted"
          >
            Không có công việc ở trạng thái này.
          </div>
          <div v-else class="space-y-3">
            <article
              v-for="note in ctrl.filteredNotes.value"
              :key="note.id"
              class="rounded-lg border border-divider bg-panel p-4 shadow-sm"
            >
              <div class="flex items-start justify-between gap-3">
                <p class="min-w-0 whitespace-pre-wrap text-sm font-semibold leading-6 text-ink">
                  {{ note.content }}
                </p>
                <div class="flex shrink-0 items-center gap-1">
                  <button
                    class="flex h-9 w-9 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
                    type="button"
                    title="Chỉnh sửa"
                    @click="openEditDialog(note)"
                  >
                    <i class="pi pi-pencil" />
                  </button>
                  <button
                    class="flex h-9 w-9 items-center justify-center rounded-md border border-red-200 bg-panel text-red-600 hover:bg-red-50"
                    type="button"
                    title="Xóa công việc"
                    @click="ctrl.removeNote(note.id)"
                  >
                    <i class="pi pi-trash" />
                  </button>
                </div>
              </div>
              <div class="mt-4 flex flex-wrap items-center gap-2">
                <button
                  v-for="status in statusOptions"
                  :key="status.value"
                  :class="[
                    'flex h-8 items-center gap-2 rounded-md border px-3 text-xs font-bold',
                    note.status === status.value
                      ? 'border-brand bg-emerald-50 text-brand'
                      : 'border-divider bg-panel text-muted hover:bg-canvas',
                  ]"
                  type="button"
                  @click="ctrl.updateNoteStatus(note.id, status.value)"
                >
                  <i :class="`pi ${status.icon} text-sm`" />
                  {{ status.label }}
                </button>
              </div>
            </article>
          </div>
        </div>
      </section>
    </section>

    <!-- Add / Edit note dialog -->
    <Dialog
      :visible="dialogVisible"
      class="w-full max-w-xl rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="dialogVisible = $event"
    >
      <template #header>
        <div>
          <h3 class="font-bold text-ink">{{ isEditing ? 'Chỉnh sửa công việc' : 'Thêm công việc' }}</h3>
          <p v-if="!isEditing" class="mt-1 text-sm text-muted">Có thể nhập ngày tương lai tối đa 1 tuần.</p>
        </div>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Ngày công việc</span>
          <Calendar
            :model-value="draftDate ? new Date(draftDate + 'T00:00:00') : null"
            class="mt-1 w-full"
            date-format="yy/mm/dd"
            placeholder="Select date"
            show-icon
            show-button-bar
            :max-date="ctrl.maxEntryDate"
            :disabled="isEditing"
            @update:model-value="draftDate = $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : ''"
          />
        </label>

        <div>
          <span class="text-xs font-bold text-muted">Trạng thái</span>
          <div class="mt-1 grid grid-cols-3 rounded-md border border-divider bg-canvas p-1">
            <button
              v-for="opt in statusOptions"
              :key="opt.value"
              :class="[
                'flex h-9 items-center justify-center gap-2 rounded-md text-sm font-bold transition',
                draftStatus === opt.value
                  ? 'bg-panel text-ink shadow-sm'
                  : 'text-muted hover:text-secondary',
              ]"
              type="button"
              @click="draftStatus = opt.value"
            >
              <i :class="`pi ${opt.icon}`" />
              <span class="truncate">{{ opt.label }}</span>
            </button>
          </div>
        </div>

        <label class="block">
          <span class="text-xs font-bold text-muted">Nội dung công việc</span>
          <textarea
            v-model="draftContent"
            class="mt-1 min-h-32 w-full resize-none rounded-md border border-divider bg-panel px-3 py-2 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            autofocus
          />
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <button
            class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
            type="button"
            @click="dialogVisible = false"
          >
            Hủy
          </button>
          <button
            class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            :disabled="!draftContent.trim() || !draftDate"
            @click="saveDraft"
          >
            {{ isEditing ? 'Cập nhật' : 'Lưu công việc' }}
          </button>
        </div>
      </template>
    </Dialog>
  </section>
</template>
