/**
 * Tính toạ độ pixel của con trỏ (caret) trong một `<textarea>`.
 *
 * Kỹ thuật "mirror div": tạo một div ẩn sao chép mọi thuộc tính style ảnh hưởng
 * tới layout của textarea, đổ nội dung tới vị trí caret, rồi đo vị trí của một
 * span đánh dấu. Toạ độ trả về tính theo gốc trên-trái của textarea (bao gồm padding),
 * chưa trừ scroll.
 */

const MIRRORED_PROPERTIES = [
  "boxSizing",
  "width",
  "borderTopWidth",
  "borderRightWidth",
  "borderBottomWidth",
  "borderLeftWidth",
  "paddingTop",
  "paddingRight",
  "paddingBottom",
  "paddingLeft",
  "fontStyle",
  "fontVariant",
  "fontWeight",
  "fontStretch",
  "fontSize",
  "fontSizeAdjust",
  "lineHeight",
  "fontFamily",
  "textAlign",
  "textTransform",
  "textIndent",
  "letterSpacing",
  "wordSpacing",
  "tabSize",
] as const;

export type CaretCoordinates = { top: number; left: number; height: number };

export function getCaretCoordinates(
  element: HTMLTextAreaElement,
  position: number,
): CaretCoordinates {
  const div = document.createElement("div");
  const style = div.style;
  const computed = window.getComputedStyle(element);

  style.position = "absolute";
  style.visibility = "hidden";
  style.whiteSpace = "pre-wrap";
  style.wordWrap = "break-word";
  style.overflow = "hidden";

  for (const prop of MIRRORED_PROPERTIES) {
    style[prop] = computed[prop];
  }

  div.textContent = element.value.substring(0, position);

  const span = document.createElement("span");
  // Ký tự đứng sau caret (hoặc "." để span có kích thước khi ở cuối chuỗi).
  span.textContent = element.value.substring(position) || ".";
  div.appendChild(span);

  document.body.appendChild(div);
  const coordinates: CaretCoordinates = {
    top: span.offsetTop + parseInt(computed.borderTopWidth || "0", 10),
    left: span.offsetLeft + parseInt(computed.borderLeftWidth || "0", 10),
    height: parseInt(computed.lineHeight || "18", 10) || 18,
  };
  document.body.removeChild(div);

  return coordinates;
}
