import { CalendarDays, CheckCircle2, ChevronLeft, ChevronRight, CircleDashed, Clock3, Plus, Trash2 } from "lucide-react";
import { Button } from "primereact/button";
import { Calendar } from "primereact/calendar";
import { Dialog } from "primereact/dialog";
import { InputTextarea } from "primereact/inputtextarea";
import { SelectButton } from "primereact/selectbutton";
import { useState } from "react";
import type { DailyWorkCalendarDay, DailyWorkNote, DailyWorkNoteDraft, DailyWorkStatus } from "../controller/useDailyWorkNotesController";

type DailyWorkNotesPageProps = {
  calendarDays: DailyWorkCalendarDay[];
  filteredNotes: DailyWorkNote[];
  maxEntryDate: Date;
  monthLabel: string;
  monthValue: string;
  onAddNote: (draft: DailyWorkNoteDraft) => boolean;
  onNextMonth: () => void;
  onPreviousMonth: () => void;
  onRemoveNote: (noteId: string) => void;
  onSelectDate: (date: string) => void;
  onSelectMonth: (value: string) => void;
  onStatusFilterChange: (status: DailyWorkStatus) => void;
  onUpdateNoteStatus: (noteId: string, status: DailyWorkStatus) => void;
  selectedDate: string;
  selectedDateLabel: string;
  statusCounts: Record<DailyWorkStatus, number>;
  statusFilter: DailyWorkStatus;
  today: Date;
  totalSelectedDateNotes: number;
};

const statusOptions: Array<{ icon: typeof CheckCircle2; label: string; value: DailyWorkStatus }> = [
  { icon: CheckCircle2, label: "Hoàn thành", value: "completed" },
  { icon: Clock3, label: "Chưa hoàn thành", value: "incomplete" },
  { icon: CircleDashed, label: "Bảo lưu", value: "reserved" },
];

const weekdays = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"];

export function DailyWorkNotesPage({
  calendarDays,
  filteredNotes,
  maxEntryDate,
  monthLabel,
  monthValue,
  onAddNote,
  onNextMonth,
  onPreviousMonth,
  onRemoveNote,
  onSelectDate,
  onSelectMonth,
  onStatusFilterChange,
  onUpdateNoteStatus,
  selectedDate,
  selectedDateLabel,
  statusCounts,
  statusFilter,
  today,
  totalSelectedDateNotes,
}: DailyWorkNotesPageProps) {
  const [isAdding, setIsAdding] = useState(false);

  return (
    <section className="grid min-h-0 flex-1 grid-cols-[minmax(360px,460px)_minmax(0,1fr)] gap-4 overflow-hidden">
      <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div className="flex h-[76px] shrink-0 items-center justify-between gap-3 border-b border-stone-200 px-4">
          <div className="flex min-w-0 items-center gap-3">
            <span className="flex h-10 w-10 items-center justify-center rounded-md bg-emerald-50 text-brand">
              <CalendarDays className="h-5 w-5" />
            </span>
            <div className="min-w-0">
              <h3 className="truncate font-bold capitalize">{monthLabel}</h3>
              <p className="mt-1 truncate text-xs text-slate-500">Tổng {totalSelectedDateNotes} công việc trong ngày đã chọn</p>
            </div>
          </div>

          <div className="flex shrink-0 items-center gap-2">
            <Button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
              type="button"
              title="Tháng trước"
              onClick={onPreviousMonth}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Calendar
              className="app-calendar h-9 w-32 rounded-md border border-slate-300 bg-white text-sm font-bold text-slate-700 outline-none hover:border-brand focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100"
              dateFormat="yy-mm"
              value={parseMonth(monthValue)}
              view="month"
              onChange={(event) => onSelectMonth(formatMonth(event.value instanceof Date ? event.value : null))}
            />
            <Button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
              type="button"
              title="Tháng sau"
              onClick={onNextMonth}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-7 border-b border-stone-200 bg-slate-800 text-center text-xs font-bold text-white">
          {weekdays.map((weekday) => (
            <div key={weekday} className="px-2 py-3">
              {weekday}
            </div>
          ))}
        </div>

        <div className="grid flex-1 grid-cols-7 grid-rows-6 bg-white">
          {calendarDays.map((day) => (
            <button
              key={day.date}
              className={[
                "min-h-0 border-b border-r border-stone-200 p-2 text-left outline-none transition focus:ring-2 focus:ring-inset focus:ring-emerald-100",
                day.isSelected ? "bg-emerald-50" : day.isCurrentMonth ? "bg-white hover:bg-slate-50" : "bg-slate-50 text-slate-400",
                day.isFutureDisabled ? "cursor-not-allowed opacity-50" : "",
              ].join(" ")}
              type="button"
              disabled={day.isFutureDisabled}
              onClick={() => onSelectDate(day.date)}
            >
              <span
                className={[
                  "flex h-7 w-7 items-center justify-center rounded-md text-sm font-extrabold tabular-nums",
                  day.isToday ? "bg-brand text-white" : day.isSelected ? "text-brand" : "text-slate-700",
                ].join(" ")}
              >
                {day.day}
              </span>
              <span className="mt-3 flex items-center justify-between gap-1">
                <span className="text-[11px] font-semibold text-slate-500">Công việc</span>
                <strong className="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-700">{day.taskCount}</strong>
              </span>
            </button>
          ))}
        </div>
      </section>

      <section className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] gap-4 overflow-hidden">
        <section className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
          <div className="flex items-center justify-between gap-3">
            <div className="min-w-0">
              <h3 className="truncate font-bold">Ghi chú công việc hằng ngày</h3>
              <p className="mt-1 truncate text-sm text-slate-500">{selectedDateLabel}</p>
            </div>
            <Button
              className="flex h-10 shrink-0 items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              onClick={() => setIsAdding(true)}
            >
              <Plus className="h-4 w-4" />
              Thêm công việc
            </Button>
          </div>
        </section>

        <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
          <div className="shrink-0 border-b border-stone-200 p-4">
            <SelectButton
              className="grid grid-cols-3 rounded-md border border-slate-200 bg-slate-100 p-1"
              value={statusFilter}
              options={statusOptions}
              optionLabel="label"
              optionValue="value"
              itemTemplate={(option) => {
                const Icon = option.icon;
                return (
                  <span className="flex h-10 items-center justify-center gap-2 text-sm font-bold">
                    <Icon className="h-4 w-4" />
                    <span className="truncate">{option.label}</span>
                    <span className="rounded-md bg-white/70 px-1.5 py-0.5 text-xs">{statusCounts[option.value]}</span>
                  </span>
                );
              }}
              allowEmpty={false}
              onChange={(event) => {
                if (event.value) {
                  onStatusFilterChange(event.value);
                }
              }}
            />
          </div>

          <div className="min-h-0 flex-1 overflow-auto p-4">
            {filteredNotes.length === 0 ? (
              <div className="flex h-full min-h-48 items-center justify-center rounded-md border border-dashed border-slate-300 bg-slate-50 px-4 text-center text-sm font-semibold text-slate-500">
                Không có công việc ở trạng thái này.
              </div>
            ) : (
              <div className="space-y-3">
                {filteredNotes.map((note) => (
                  <DailyWorkNoteItem
                    key={note.id}
                    note={note}
                    onRemove={onRemoveNote}
                    onUpdateStatus={onUpdateNoteStatus}
                  />
                ))}
              </div>
            )}
          </div>
        </section>
      </section>

      {isAdding ? (
        <AddDailyWorkNoteDialog
          maxEntryDate={maxEntryDate}
          selectedDate={selectedDate}
          today={today}
          onClose={() => setIsAdding(false)}
          onSave={(draft) => {
            if (onAddNote(draft)) {
              setIsAdding(false);
            }
          }}
        />
      ) : null}
    </section>
  );
}

function DailyWorkNoteItem({
  note,
  onRemove,
  onUpdateStatus,
}: {
  note: DailyWorkNote;
  onRemove: (noteId: string) => void;
  onUpdateStatus: (noteId: string, status: DailyWorkStatus) => void;
}) {
  return (
    <article className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
      <div className="flex items-start justify-between gap-3">
        <p className="min-w-0 whitespace-pre-wrap text-sm font-semibold leading-6 text-slate-800">{note.content}</p>
        <Button
          className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-red-200 bg-white text-red-600 hover:bg-red-50"
          type="button"
          title="Xóa công việc"
          onClick={() => onRemove(note.id)}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-2">
        {statusOptions.map((status) => {
          const Icon = status.icon;
          const isActive = note.status === status.value;
          return (
            <Button
              key={status.value}
              className={[
                "flex h-8 items-center gap-2 rounded-md border px-3 text-xs font-bold",
                isActive ? "border-brand bg-emerald-50 text-brand" : "border-slate-200 bg-white text-slate-500 hover:bg-slate-50",
              ].join(" ")}
              type="button"
              onClick={() => onUpdateStatus(note.id, status.value)}
            >
              <Icon className="h-3.5 w-3.5" />
              {status.label}
            </Button>
          );
        })}
      </div>
    </article>
  );
}

function AddDailyWorkNoteDialog({
  maxEntryDate,
  onClose,
  onSave,
  selectedDate,
  today,
}: {
  maxEntryDate: Date;
  onClose: () => void;
  onSave: (draft: DailyWorkNoteDraft) => void;
  selectedDate: string;
  today: Date;
}) {
  const initialDate = parseDate(selectedDate) ?? today;
  const [form, setForm] = useState<DailyWorkNoteDraft>({
    content: "",
    date: initialDate > maxEntryDate ? today : initialDate,
    status: "completed",
  });

  return (
    <Dialog
      className="w-full max-w-xl rounded-lg bg-white shadow-xl"
      contentClassName="p-0"
      header={
        <div>
          <h3 className="font-bold text-slate-900">Thêm công việc</h3>
          <p className="mt-1 text-sm text-slate-500">Có thể nhập ngày tương lai tối đa 1 tuần.</p>
        </div>
      }
      visible
      onHide={onClose}
    >
      <div className="space-y-4 px-5 py-4">
        <label className="block">
          <span className="text-xs font-bold text-slate-500">Ngày công việc</span>
          <Calendar
            className="app-calendar mt-1 h-10 w-full rounded-md border border-slate-300 bg-white text-sm text-slate-900 outline-none focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100"
            dateFormat="yy-mm-dd"
            maxDate={maxEntryDate}
            value={form.date}
            onChange={(event) => setForm((current) => ({ ...current, date: event.value instanceof Date ? event.value : null }))}
          />
        </label>

        <label className="block">
          <span className="text-xs font-bold text-slate-500">Trạng thái</span>
          <SelectButton
            className="mt-1 grid grid-cols-3 rounded-md border border-slate-200 bg-slate-100 p-1"
            value={form.status}
            options={statusOptions}
            optionLabel="label"
            optionValue="value"
            allowEmpty={false}
            itemTemplate={(option) => {
              const Icon = option.icon;
              return (
                <span className="flex h-9 items-center justify-center gap-2 text-sm font-bold">
                  <Icon className="h-4 w-4" />
                  <span className="truncate">{option.label}</span>
                </span>
              );
            }}
            onChange={(event) => {
              if (event.value) {
                setForm((current) => ({ ...current, status: event.value }));
              }
            }}
          />
        </label>

        <label className="block">
          <span className="text-xs font-bold text-slate-500">Nội dung công việc</span>
          <InputTextarea
            className="mt-1 min-h-32 w-full resize-none rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            autoFocus
            value={form.content}
            onChange={(event) => setForm((current) => ({ ...current, content: event.target.value }))}
          />
        </label>
      </div>

      <div className="flex items-center justify-end gap-2 border-t border-stone-200 px-5 py-4">
        <Button
          className="h-10 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
          type="button"
          onClick={onClose}
        >
          Hủy
        </Button>
        <Button
          className="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
          type="button"
          disabled={!form.content.trim() || !form.date}
          onClick={() => onSave(form)}
        >
          Lưu công việc
        </Button>
      </div>
    </Dialog>
  );
}

function parseMonth(value: string) {
  const [year, month] = value.split("-").map(Number);
  if (!year || !month) {
    return null;
  }

  return new Date(year, month - 1, 1);
}

function formatMonth(value: Date | null) {
  if (!value) {
    return "";
  }

  return `${value.getFullYear()}-${String(value.getMonth() + 1).padStart(2, "0")}`;
}

function parseDate(value: string) {
  const [year, month, day] = value.split("-").map(Number);
  if (!year || !month || !day) {
    return null;
  }

  return new Date(year, month - 1, day);
}
