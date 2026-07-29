import type { IWorkbookData } from "@univerjs/core";
import { createUniver, LocaleType, UniverInstanceType } from "@univerjs/presets";
import { UniverSheetsCorePreset } from "@univerjs/preset-sheets-core";
import sheetsCoreEnUS from "@univerjs/preset-sheets-core/locales/en-US";
import "@univerjs/preset-sheets-core/lib/index.css";
import { UniverSheetsDrawingPreset } from "@univerjs/preset-sheets-drawing";
import sheetsDrawingEnUS from "@univerjs/preset-sheets-drawing/locales/en-US";
import "@univerjs/preset-sheets-drawing/lib/index.css";
import LuckyExcel from "@zwight/luckyexcel";
import { onBeforeUnmount, ref } from "vue";
import { base64ToBytes } from "@/shared/utils/base64";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { explorerReadFileBase64 } from "@/tauri/commands/explorer";

function fileNameFromPath(path: string) {
  return path.split(/[\\/]/).pop() || "workbook.xlsx";
}

// Univer vẽ ảnh (drawImage) lên canvas mà không tự set imageSmoothingQuality, nên trình duyệt
// dùng bộ lọc resample mặc định (khá thô) khi ảnh evidence lớn bị co lại vừa vài ô — nhìn vỡ/nhòe
// hơn hẳn so với Excel. Patch 1 lần duy nhất tại nguồn (HTMLCanvasElement.prototype.getContext)
// để mọi canvas 2D mà Univer tạo ra (kể cả khi resize/re-render) đều được bật smoothing chất
// lượng cao ngay khi khởi tạo.
let canvasSmoothingPatched = false;
function ensureHighQualityCanvasSmoothing() {
  if (canvasSmoothingPatched || typeof HTMLCanvasElement === "undefined") return;
  canvasSmoothingPatched = true;
  const originalGetContext = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = function (this: HTMLCanvasElement, ...args: Parameters<typeof originalGetContext>) {
    const ctx = originalGetContext.apply(this, args);
    if (ctx && args[0] === "2d" && "imageSmoothingQuality" in ctx) {
      const ctx2d = ctx as CanvasRenderingContext2D;
      ctx2d.imageSmoothingEnabled = true;
      ctx2d.imageSmoothingQuality = "high";
    }
    return ctx;
  } as typeof originalGetContext;
}

// `imageSmoothingQuality: "high"` chỉ là 1 lượt lọc bilinear/bicubic đơn — khi 1 ảnh chụp màn
// hình lớn bị co lại nhiều lần (>2x, rất thường gặp với evidence dán vào vài ô) trong 1 lần vẽ,
// kết quả vẫn nhòe hơn hẳn cách Excel/GDI resample. Khắc phục bằng kỹ thuật mipmap thủ công: co
// dần ảnh nguồn từng nấc (mỗi nấc tối đa 2x, luôn dùng smoothing "high") qua các canvas ẩn cho
// tới khi gần bằng kích thước đích, rồi mới vẽ nấc cuối lên canvas thật — cho ảnh nét hơn hẳn so
// với co 1 lần duy nhất. Cache theo từng ảnh nguồn (WeakMap) để không phải tính lại mỗi khi
// Univer render lại (scroll/pan) — cache tự giải phóng khi ảnh gốc bị dispose.
interface DownscaledSource {
  canvas: HTMLCanvasElement;
  width: number;
  height: number;
}

const downscaleCache = new WeakMap<CanvasImageSource, Map<string, DownscaledSource>>();

function sourcePixelSize(image: CanvasImageSource): { width: number; height: number } | null {
  if (image instanceof HTMLImageElement) return { width: image.naturalWidth, height: image.naturalHeight };
  if (image instanceof HTMLCanvasElement || image instanceof HTMLVideoElement) return { width: image.width, height: image.height };
  if (typeof ImageBitmap !== "undefined" && image instanceof ImageBitmap) return { width: image.width, height: image.height };
  return null;
}

function getProgressivelyDownscaled(
  image: CanvasImageSource,
  sx: number,
  sy: number,
  sw: number,
  sh: number,
  targetWidth: number,
  targetHeight: number,
): DownscaledSource | null {
  if (sw <= 0 || sh <= 0 || targetWidth <= 0 || targetHeight <= 0) return null;
  if (!sourcePixelSize(image)) return null;

  const key = `${sx},${sy},${sw},${sh}->${targetWidth}x${targetHeight}`;
  let cacheForImage = downscaleCache.get(image);
  const cached = cacheForImage?.get(key);
  if (cached) return cached;

  let stepSource: CanvasImageSource = image;
  let stepSx = sx;
  let stepSy = sy;
  let stepWidth = sw;
  let stepHeight = sh;
  let lastStepCanvas: HTMLCanvasElement | null = null;

  while (stepWidth > targetWidth * 2 || stepHeight > targetHeight * 2) {
    const nextWidth = Math.max(targetWidth, Math.round(stepWidth / 2));
    const nextHeight = Math.max(targetHeight, Math.round(stepHeight / 2));
    const stepCanvas = document.createElement("canvas");
    stepCanvas.width = nextWidth;
    stepCanvas.height = nextHeight;
    const stepCtx = stepCanvas.getContext("2d");
    if (!stepCtx) return null;
    stepCtx.imageSmoothingEnabled = true;
    stepCtx.imageSmoothingQuality = "high";
    stepCtx.drawImage(stepSource, stepSx, stepSy, stepWidth, stepHeight, 0, 0, nextWidth, nextHeight);

    stepSource = stepCanvas;
    stepSx = 0;
    stepSy = 0;
    stepWidth = nextWidth;
    stepHeight = nextHeight;
    lastStepCanvas = stepCanvas;
  }

  if (!lastStepCanvas) return null;
  const result: DownscaledSource = { canvas: lastStepCanvas, width: stepWidth, height: stepHeight };
  if (!cacheForImage) {
    cacheForImage = new Map();
    downscaleCache.set(image, cacheForImage);
  }
  cacheForImage.set(key, result);
  return result;
}

let drawImagePatched = false;
function ensureProgressiveImageDownscaling() {
  if (drawImagePatched || typeof CanvasRenderingContext2D === "undefined") return;
  drawImagePatched = true;
  const originalDrawImage = CanvasRenderingContext2D.prototype.drawImage;
  CanvasRenderingContext2D.prototype.drawImage = function (this: CanvasRenderingContext2D, ...args: unknown[]) {
    const isUniverCanvas = (this.canvas as HTMLCanvasElement | undefined)?.dataset?.uComp === "render-canvas";
    if (isUniverCanvas && (args.length === 5 || args.length === 9)) {
      const image = args[0] as CanvasImageSource;
      const size = sourcePixelSize(image);
      let sx = 0;
      let sy = 0;
      let sw = size?.width ?? 0;
      let sh = size?.height ?? 0;
      let dx: number;
      let dy: number;
      let dw: number;
      let dh: number;
      if (args.length === 9) {
        [, sx, sy, sw, sh, dx, dy, dw, dh] = args as [unknown, number, number, number, number, number, number, number, number];
      } else {
        [, dx, dy, dw, dh] = args as [unknown, number, number, number, number];
      }
      if (sw > 0 && sh > 0 && dw > 0 && dh > 0 && (dw < sw * 0.6 || dh < sh * 0.6)) {
        const downscaled = getProgressivelyDownscaled(image, sx, sy, sw, sh, Math.round(dw), Math.round(dh));
        if (downscaled) {
          originalDrawImage.call(this, downscaled.canvas, 0, 0, downscaled.width, downscaled.height, dx, dy, dw, dh);
          return;
        }
      }
    }
    (originalDrawImage as (...a: unknown[]) => void).apply(this, args);
  } as typeof originalDrawImage;
}

function ensureCrispEvidenceImageRendering() {
  ensureHighQualityCanvasSmoothing();
  ensureProgressiveImageDownscaling();
}

export function useExcelPreview() {
  const containerRef = ref<HTMLElement | null>(null);
  const isLoading = ref(false);
  const errorMessage = ref("");

  let disposeCurrent: (() => void) | null = null;

  function renderWorkbook(data: IWorkbookData) {
    if (!containerRef.value) return;
    disposeCurrent?.();
    disposeCurrent = null;
    ensureCrispEvidenceImageRendering();

    const { univer } = createUniver({
      locale: LocaleType.EN_US,
      locales: {
        [LocaleType.EN_US]: {
          ...sheetsCoreEnUS,
          ...sheetsDrawingEnUS,
        },
      },
      presets: [UniverSheetsCorePreset({ container: containerRef.value }), UniverSheetsDrawingPreset()],
    });
    disposeCurrent = () => univer.dispose();
    univer.createUnit(UniverInstanceType.UNIVER_SHEET, data);
  }

  async function loadWorkbook(path: string) {
    if (!path.trim() || !canUseTauriRuntime()) return;
    isLoading.value = true;
    errorMessage.value = "";
    try {
      const base64 = await explorerReadFileBase64(path);
      const bytes = base64ToBytes(base64);
      const file = new File([bytes as BlobPart], fileNameFromPath(path), {
        type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      });
      await new Promise<void>((resolve) => {
        LuckyExcel.transformExcelToUniver(
          file,
          (workbookData: IWorkbookData) => {
            try {
              renderWorkbook(workbookData);
            } catch (e) {
              errorMessage.value = friendlyError(e);
            }
            resolve();
          },
          (err: Error) => {
            errorMessage.value = err.message || "Could not parse this workbook for preview.";
            resolve();
          },
        );
      });
    } catch (e) {
      errorMessage.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  function clear() {
    disposeCurrent?.();
    disposeCurrent = null;
    errorMessage.value = "";
    isLoading.value = false;
  }

  onBeforeUnmount(() => {
    disposeCurrent?.();
    disposeCurrent = null;
  });

  return { containerRef, isLoading, errorMessage, loadWorkbook, clear };
}
