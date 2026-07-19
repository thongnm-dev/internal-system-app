import { computed, ref, shallowRef } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { canUseTauriRuntime } from "@/tauri/commands/_base";

/**
 * Trạng thái vòng đời của bản cập nhật hiển thị trên thanh trạng thái.
 * - `idle`        : chưa có bản cập nhật (hoặc chưa kiểm tra).
 * - `checking`    : đang kiểm tra release mới.
 * - `downloading` : đang tải tự động, kèm phần trăm.
 * - `ready`       : tải xong, sẵn sàng cài đặt (chờ user click).
 * - `installing`  : đang cài đặt và khởi động lại.
 * - `error`       : có lỗi trong quá trình kiểm tra/tải/cài.
 */
export type UpdaterStatus =
  | "idle"
  | "checking"
  | "downloading"
  | "ready"
  | "installing"
  | "error";

/**
 * Quản lý luồng auto-update:
 * kiểm tra release -> tự động tải (hiển thị %) -> sẵn sàng -> user click để cài.
 *
 * Trả về state phản ứng để `AppBottomBar` hiển thị và hàm `install()`
 * để cài đặt khi user bấm vào nhãn "Bản cập nhật sẵn sàng.".
 */
export function useAppUpdater() {
  const status = ref<UpdaterStatus>("idle");
  const version = ref<string | null>(null);
  const errorMessage = ref<string | null>(null);

  const downloadedBytes = ref(0);
  const totalBytes = ref(0);

  // Giữ tham chiếu tới bản cập nhật đã tải để cài khi user bấm.
  // Không dùng reactive để tránh proxy hoá Resource của Tauri.
  const pending = shallowRef<Update | null>(null);

  /** Phần trăm tải (0-100). Trả về null khi chưa biết tổng dung lượng. */
  const downloadPercent = computed(() => {
    if (totalBytes.value <= 0) {
      return null;
    }
    const pct = Math.round((downloadedBytes.value / totalBytes.value) * 100);
    return Math.min(100, Math.max(0, pct));
  });

  /** True khi có bản cập nhật đang trong luồng xử lý (tải / sẵn sàng / cài). */
  const isActive = computed(
    () =>
      status.value === "checking" ||
      status.value === "downloading" ||
      status.value === "ready" ||
      status.value === "installing",
  );

  async function downloadUpdate(update: Update): Promise<void> {
    status.value = "downloading";
    downloadedBytes.value = 0;
    totalBytes.value = 0;

    await update.download((event) => {
      switch (event.event) {
        case "Started":
          totalBytes.value = event.data.contentLength ?? 0;
          downloadedBytes.value = 0;
          break;
        case "Progress":
          downloadedBytes.value += event.data.chunkLength;
          break;
        case "Finished":
          if (totalBytes.value > 0) {
            downloadedBytes.value = totalBytes.value;
          }
          break;
      }
    });

    pending.value = update;
    status.value = "ready";
  }

  /**
   * Kiểm tra release mới. Nếu có, tự động tải ngay và hiển thị phần trăm.
   * Gọi một lần khi app khởi động (và có thể gọi lại định kỳ nếu cần).
   */
  async function checkAndDownload(): Promise<void> {
    if (!canUseTauriRuntime()) {
      return;
    }
    // Đang trong luồng xử lý thì không kiểm tra chồng chéo.
    if (isActive.value) {
      return;
    }

    status.value = "checking";
    errorMessage.value = null;

    try {
      const update = await check();
      if (!update) {
        status.value = "idle";
        version.value = null;
        return;
      }

      version.value = update.version;
      await downloadUpdate(update);
    } catch (error) {
      errorMessage.value = String(error);
      status.value = "error";
    }
  }

  /**
   * Cài đặt bản cập nhật đã tải rồi khởi động lại ứng dụng.
   * Gọi khi user bấm vào nhãn "Bản cập nhật sẵn sàng.".
   */
  async function install(): Promise<void> {
    const update = pending.value;
    if (!update || status.value !== "ready") {
      return;
    }

    status.value = "installing";
    try {
      await update.install();
      await relaunch();
    } catch (error) {
      errorMessage.value = String(error);
      status.value = "error";
    }
  }

  return {
    status,
    version,
    errorMessage,
    downloadPercent,
    downloadedBytes,
    totalBytes,
    isActive,
    checkAndDownload,
    install,
  };
}
