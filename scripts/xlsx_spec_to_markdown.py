#!/usr/bin/env python3
"""Convert the KUKM screen design workbook into linked Markdown.

The converter intentionally uses only Python's standard library so it can run
without installing Excel-specific packages. It reads the XLSX package directly
as ZIP/XML, extracts cell text, and rebuilds the workbook's call-number links
as Markdown anchors.
"""

from __future__ import annotations

import argparse
import html
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable
from zipfile import ZipFile
import xml.etree.ElementTree as ET


MAIN_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL_NS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
NS = {"m": MAIN_NS, "r": REL_NS}

EVENT_SHEET = "イベント詳細設計書"
EVENT_CONTENT_START_ROW = 4
EVENT_CONTENT_START_COL = "D"
EVENT_CONTENT_END_COL = "AG"
EVENT_REF_SHEET_COL = "AH"
EVENT_REF_NO_COL = "AI"
EVENT_NOTE_COL = "AJ"
IF_EDIT_SHEET = "IF編集仕様"
SCREEN_EDIT_SHEET = "画面編集要領"
SQL_PARAM_SHEET = "SQLパラメータ編集仕様"
CHANGE_HISTORY_SHEET = "変更履歴"
SCREEN_ITEMS_SHEET = "画面部品説明書(画面項目)"
INTERNAL_PARAMS_SHEET = "内部パラメータ"
CHECK_SPEC_SHEET = "チェック仕様"
EXCLUDED_VISIBLE_SHEETS = {"プログラム処理概要図"}


@dataclass(frozen=True)
class SheetInfo:
    index: int
    name: str
    path: str
    dimension: str


@dataclass
class SheetData:
    info: SheetInfo
    cells: dict[tuple[int, int], str]
    merged_cells: dict[tuple[int, int], tuple[int, int]]
    max_row: int
    max_col: int


def main() -> None:
    parser = argparse.ArgumentParser(description="Convert screen design XLSX to Markdown.")
    parser.add_argument("input", type=Path, help="Input .xlsx file")
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Output .md file. Defaults to the input filename with .md extension.",
    )
    args = parser.parse_args()

    output_path = args.output or args.input.with_suffix(".md")
    workbook = XlsxWorkbook(args.input)
    sheets = workbook.read_sheets()
    call_anchor_index = build_call_anchor_index(sheets)
    markdown = render_markdown(args.input, sheets, call_anchor_index)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(markdown, encoding="utf-8")
    print(output_path)


class XlsxWorkbook:
    def __init__(self, path: Path) -> None:
        self.path = path

    def read_sheets(self) -> list[SheetData]:
        with ZipFile(self.path) as archive:
            shared_strings = read_shared_strings(archive)
            sheet_infos = read_sheet_infos(archive)
            return [read_sheet_data(archive, info, shared_strings) for info in sheet_infos]


def read_shared_strings(archive: ZipFile) -> list[str]:
    if "xl/sharedStrings.xml" not in archive.namelist():
        return []

    root = ET.fromstring(archive.read("xl/sharedStrings.xml"))
    strings: list[str] = []
    for item in root.findall("m:si", NS):
        strings.append("".join(text.text or "" for text in item.findall(".//m:t", NS)))
    return strings


def read_sheet_infos(archive: ZipFile) -> list[SheetInfo]:
    workbook = ET.fromstring(archive.read("xl/workbook.xml"))
    rels = ET.fromstring(archive.read("xl/_rels/workbook.xml.rels"))
    rel_targets = {rel.attrib["Id"]: rel.attrib["Target"] for rel in rels}

    infos: list[SheetInfo] = []
    for index, sheet in enumerate(workbook.find("m:sheets", NS) or [], start=1):
        name = sheet.attrib["name"]
        if sheet.attrib.get("state", "visible") != "visible" or name in EXCLUDED_VISIBLE_SHEETS:
            continue
        rel_id = sheet.attrib[f"{{{REL_NS}}}id"]
        target = rel_targets[rel_id].lstrip("/")
        path = f"xl/{target}"
        root = ET.fromstring(archive.read(path))
        dimension = ""
        dimension_node = root.find("m:dimension", NS)
        if dimension_node is not None:
            dimension = dimension_node.attrib.get("ref", "")
        infos.append(SheetInfo(index=index, name=name, path=path, dimension=dimension))
    return infos


def read_sheet_data(archive: ZipFile, info: SheetInfo, shared_strings: list[str]) -> SheetData:
    root = ET.fromstring(archive.read(info.path))
    cells: dict[tuple[int, int], str] = {}
    merged_cells = read_merged_cells(root)
    max_row = 0
    max_col = 0

    for cell in root.findall(".//m:c", NS):
        ref = cell.attrib.get("r")
        if not ref:
            continue
        col, row = split_cell_ref(ref)
        value = read_cell_value(cell, shared_strings)
        if value:
            cells[(row, col)] = normalize_cell_text(value)
            max_row = max(max_row, row)
            max_col = max(max_col, col)

    dim_max_row, dim_max_col = max_from_dimension(info.dimension)
    max_row = max(max_row, dim_max_row)
    max_col = max(max_col, dim_max_col)
    return SheetData(info=info, cells=cells, merged_cells=merged_cells, max_row=max_row, max_col=max_col)


def read_cell_value(cell: ET.Element, shared_strings: list[str]) -> str:
    cell_type = cell.attrib.get("t")
    value_node = cell.find("m:v", NS)

    if cell_type == "s":
        if value_node is None or value_node.text is None:
            return ""
        index = int(value_node.text)
        return shared_strings[index] if 0 <= index < len(shared_strings) else ""

    if cell_type == "inlineStr":
        return "".join(text.text or "" for text in cell.findall(".//m:t", NS))

    if value_node is None or value_node.text is None:
        return ""
    return value_node.text


def render_markdown(
    input_path: Path,
    sheets: list[SheetData],
    call_anchor_index: dict[tuple[str, str], str],
) -> str:
    lines = [
        f"# {input_path.name}",
        "",
        "> Generated from Excel screen design workbook.",
        "",
        "## Sheet Index",
        "",
    ]

    for sheet in sheets:
        lines.append(f"- [{sheet.info.name}](#{sheet_anchor(sheet.info)})")
    lines.append("")

    for sheet in sheets:
        lines.extend(render_sheet(sheet, call_anchor_index))
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def render_sheet(sheet: SheetData, call_anchor_index: dict[tuple[str, str], str]) -> list[str]:
    lines = [f"## {anchor_tag(sheet_anchor(sheet.info))}{escape_md(sheet.info.name)}", ""]

    if sheet.info.name == EVENT_SHEET:
        lines.extend(render_event_sheet(sheet, call_anchor_index))
        return lines

    if sheet.info.name == CHANGE_HISTORY_SHEET:
        lines.extend(render_change_history_sheet(sheet))
        return lines

    if sheet.info.name == SCREEN_ITEMS_SHEET:
        lines.extend(render_screen_items_sheet(sheet))
        return lines

    if sheet.info.name == INTERNAL_PARAMS_SHEET:
        lines.extend(render_internal_params_sheet(sheet))
        return lines

    if sheet.info.name == IF_EDIT_SHEET:
        lines.extend(
            render_call_table_sheet(
                sheet,
                [
                    ("インタフェース項目名称または内部パラメータ項目名称", "D"),
                    ("編集元", "E"),
                    ("編集内容", "F"),
                    ("補足または補足項番", "G"),
                ],
            ),
        )
        return lines

    if sheet.info.name == SCREEN_EDIT_SHEET:
        lines.extend(
            render_call_table_sheet(
                sheet,
                [
                    ("画面項目名称", "D"),
                    ("部品の種類", "E"),
                    ("部品の表示/非表示", "F"),
                    ("部品の活性/非活性", "G"),
                    ("表示内容", "H"),
                    ("リストデータ取得元", "I"),
                    ("表示内容取得元", "J"),
                    ("出力編集仕様", "K"),
                    ("補足または補足項番", "L"),
                ],
            ),
        )
        return lines

    if sheet.info.name == SQL_PARAM_SHEET:
        lines.extend(
            render_call_table_sheet(
                sheet,
                [
                    ("SQLパラメータ項目名称", "D"),
                    ("編集元", "E"),
                    ("編集内容", "F"),
                    ("補足または補足項番", "G"),
                ],
                title_extra_col="B",
            ),
        )
        return lines

    if sheet.info.name == CHECK_SPEC_SHEET:
        lines.extend(
            render_call_table_sheet(
                sheet,
                [
                    ("画面項目識別ID(自)", "C"),
                    ("チェック対象項目名称(自)", "D"),
                    ("部品の種類(自)", "E"),
                    ("画面項目識別ID(至)", "F"),
                    ("チェック対象項目名称(至)", "G"),
                    ("部品の種類(至)", "H"),
                    ("必須", "I"),
                    ("最小値/最小文字数/最小バイト数", "J"),
                    ("最大値/最大文字数/最大バイト数", "K"),
                    ("比較", "L"),
                    ("その他チェック内容", "M"),
                    ("エラー種別", "N"),
                    ("メッセージID", "O"),
                    ("パラメータ{0}", "P"),
                    ("パラメータ{1}", "Q"),
                    ("補足または補足項番", "R"),
                ],
            ),
        )
        return lines

    lines.extend(render_table_sheet(sheet))
    return lines


def render_event_sheet(sheet: SheetData, call_anchor_index: dict[tuple[str, str], str]) -> list[str]:
    start_col = col_to_num(EVENT_CONTENT_START_COL)
    end_col = col_to_num(EVENT_CONTENT_END_COL)
    ref_sheet_col = col_to_num(EVENT_REF_SHEET_COL)
    ref_no_col = col_to_num(EVENT_REF_NO_COL)
    note_col = col_to_num(EVENT_NOTE_COL)
    lines: list[str] = []
    block_starts = find_event_block_starts(sheet)

    for index, start_row in enumerate(block_starts):
        end_row = block_starts[index + 1] - 1 if index + 1 < len(block_starts) else sheet.max_row
        method_name = event_meta_value(sheet, start_row, col_to_num("B"))
        event_name = event_meta_value(sheet, start_row, col_to_num("C"))
        title = method_name if method_name != "-" else event_name

        if lines:
            lines.append("")
        lines.append(f"## {escape_md(title)}")
        lines.append("")
        lines.append(f"- メソッド名称: {escape_md(method_name)}")
        lines.append(f"- イベント名称: {escape_md(event_name)}")
        lines.append("- 処理詳細:  ")

        detail_lines = []
        for row in range(start_row, end_row + 1):
            parts = [
                (col, sheet.cells.get((row, col), ""))
                for col in range(start_col, end_col + 1)
                if sheet.cells.get((row, col), "")
            ]
            if not parts:
                continue

            ref_sheet = sheet.cells.get((row, ref_sheet_col), "")
            ref_no = normalize_call_no(sheet.cells.get((row, ref_no_col), ""))
            reference = render_reference(ref_sheet, ref_no, call_anchor_index)
            note = sheet.cells.get((row, note_col), "")
            detail_lines.append(render_event_detail_line(parts, start_col, reference, note))

        if detail_lines:
            lines.extend(detail_lines)
        else:
            lines.append("  -")

    return lines


def render_table_sheet(sheet: SheetData) -> list[str]:
    used_rows = sorted({row for row, _ in sheet.cells})
    if not used_rows:
        return ["_No content._"]

    lines: list[str] = []
    anchored_call_numbers: set[str] = set()

    for row in used_rows:
        values = [value for _, value in sorted((col, sheet.cells[(row, col)]) for cell_row, col in sheet.cells if cell_row == row)]
        if not values:
            continue
        call_no = normalize_call_no(sheet.cells.get((row, 1), ""))
        anchor = ""
        if call_no and call_no not in anchored_call_numbers:
            anchor = anchor_tag(call_anchor(sheet.info, call_no))
            anchored_call_numbers.add(call_no)

        content = " / ".join(escape_md(value).replace("\n", "<br>") for value in values)
        lines.append(f"- {anchor}{content}")

    return lines or ["_No content._"]


def render_change_history_sheet(sheet: SheetData) -> list[str]:
    rows = [["変更日", "変更者所属", "変更者名前", "新規作成／変更箇所", "変更理由"]]
    for row in range(4, sheet.max_row + 1):
        values = [sheet.cells.get((row, col), "") for col in range(1, 6)]
        if any(values):
            rows.append(values)
    return markdown_table(rows)


def render_screen_items_sheet(sheet: SheetData) -> list[str]:
    rows = [
        [
            "識別ID",
            "項目No",
            "項目内連番",
            "画面項目名称",
            "部品の種類",
            "部品の型",
            "対応DB項目名称",
            "作業エリア",
            "計算エリア/重複チェック",
            "項目記号名称",
            "タブ順",
            "フォーカス",
            "空白行要否",
            "手入力変更可否",
            "Text Align",
            "備考",
        ],
    ]
    columns = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "K", "L", "M", "N", "O", "P", "Q"]
    for row in range(7, sheet.max_row + 1):
        values = [sheet.cells.get((row, col_to_num(col)), "") for col in columns]
        if any(values):
            rows.append(values)
    return markdown_table(rows)


def render_internal_params_sheet(sheet: SheetData) -> list[str]:
    sections = [
        ("内部変数一覧", 7, 8, 12, ["No", "区分", "記号名称", "名称", "スコープ", "型", "概要", "備考"], list("ABCDEFGH")),
        ("画面固有設定値一覧", 19, 20, 24, ["No", "名称", "概要", "備考"], list("ABCD")),
        ("イベント一覧", 27, 28, 31, ["No", "画面項目名称", "イベント名称", "概要", "備考"], ["A", "B", "D", "G", "H"]),
        ("メソッド一覧", 39, 40, sheet.max_row, ["No", "区分", "記号名称", "名称", "スコープ", "型", "概要", "備考"], list("ABCDEFGH")),
    ]
    lines: list[str] = []
    for title, _header_row, start_row, end_row, headers, columns in sections:
        rows = [headers]
        for row in range(start_row, end_row + 1):
            values = [sheet.cells.get((row, col_to_num(col)), "") for col in columns]
            if any(values):
                rows.append(values)
        if len(rows) == 1:
            continue
        if lines:
            lines.append("")
        lines.append(f"### {escape_md(title)}")
        lines.append("")
        lines.extend(markdown_table(rows))
    return lines or ["_No content._"]


def render_call_table_sheet(
    sheet: SheetData,
    columns: list[tuple[str, str]],
    title_extra_col: str | None = None,
) -> list[str]:
    lines: list[str] = []
    call_rows = find_call_rows(sheet)

    for call_no, start_row, end_row in call_rows:
        anchor = call_anchor(sheet.info, call_no)
        title = f"{sheet.info.name}_{call_no}"
        if title_extra_col:
            extra = merged_cell_value(sheet, start_row, col_to_num(title_extra_col)).replace("\n", "")
            if extra:
                title = f"{title} `{escape_md(extra)}`"

        if lines:
            lines.append("")
        lines.append(f"## {anchor_tag(anchor)}{title}")
        lines.append("")

        table_rows = [[header for header, _ in columns]]
        for row in range(start_row + 1, end_row + 1):
            values = [sheet.cells.get((row, col_to_num(col)), "") for _, col in columns]
            if any(value for value in values):
                table_rows.append(values)

        lines.extend(markdown_table(table_rows))

    return lines or ["_No content._"]


def find_call_rows(sheet: SheetData) -> list[tuple[str, int, int]]:
    starts: list[tuple[str, int]] = []
    seen_call_numbers: set[str] = set()
    no_col = col_to_num("A")
    detail_no_col = col_to_num("C")

    for row in range(1, sheet.max_row + 1):
        call_no = normalize_call_no(sheet.cells.get((row, no_col), ""))
        detail_no = sheet.cells.get((row, detail_no_col), "")
        if call_no and call_no != "000" and not detail_no and call_no not in seen_call_numbers:
            starts.append((call_no, row))
            seen_call_numbers.add(call_no)

    ranges: list[tuple[str, int, int]] = []
    for index, (call_no, start_row) in enumerate(starts):
        end_row = starts[index + 1][1] - 1 if index + 1 < len(starts) else sheet.max_row
        ranges.append((call_no, start_row, end_row))
    return ranges


def render_reference(ref_sheet: str, ref_no: str, call_anchor_index: dict[tuple[str, str], str]) -> str:
    if not ref_sheet or not ref_no:
        return ""

    anchor = call_anchor_index.get((ref_sheet, ref_no))
    label = f"{ref_sheet} ({ref_no})"
    if not anchor:
        return escape_md(label)
    return f"[{escape_md(label)}](#{anchor})"


def build_call_anchor_index(sheets: Iterable[SheetData]) -> dict[tuple[str, str], str]:
    index: dict[tuple[str, str], str] = {}
    for sheet in sheets:
        if sheet.info.name in {IF_EDIT_SHEET, SCREEN_EDIT_SHEET, SQL_PARAM_SHEET, CHECK_SPEC_SHEET}:
            for call_no, _, _ in find_call_rows(sheet):
                index[(sheet.info.name, call_no)] = call_anchor(sheet.info, call_no)
        else:
            for row in range(1, sheet.max_row + 1):
                call_no = normalize_call_no(sheet.cells.get((row, 1), ""))
                if call_no and (sheet.info.name, call_no) not in index:
                    index[(sheet.info.name, call_no)] = call_anchor(sheet.info, call_no)
    return index


def find_event_block_starts(sheet: SheetData) -> list[int]:
    starts: list[int] = []
    method_col = col_to_num("B")
    event_col = col_to_num("C")

    for row in range(EVENT_CONTENT_START_ROW + 1, sheet.max_row + 1):
        has_raw_method_or_event = bool(sheet.cells.get((row, method_col)) or sheet.cells.get((row, event_col)))
        if has_raw_method_or_event:
            starts.append(row)
    return starts


def event_meta_value(sheet: SheetData, row: int, col: int) -> str:
    value = merged_cell_value(sheet, row, col).replace("\n", "")
    value = value.replace(" / ", "").replace("/", "").strip()
    return value or "-"


def render_event_detail_line(parts: list[tuple[int, str]], start_col: int, reference: str, note: str) -> str:
    first_col = parts[0][0]
    indent_level = max(0, min(first_col - start_col, 4))
    text = "".join(value.replace("\n", "") for _, value in parts).strip()
    if reference:
        text = f"{text}{reference}"
    if note:
        text = f"{text} `{note.replace(chr(10), ' ')}`"
    indent = "&nbsp;" * (indent_level * 4 + 4)
    return f"  {indent}{escape_md(text)}  "


def merged_cell_value(sheet: SheetData, row: int, col: int) -> str:
    value = sheet.cells.get((row, col), "")
    if value:
        return value
    source = sheet.merged_cells.get((row, col))
    if not source:
        return ""
    return sheet.cells.get(source, "")


def read_merged_cells(root: ET.Element) -> dict[tuple[int, int], tuple[int, int]]:
    merged_cells: dict[tuple[int, int], tuple[int, int]] = {}
    merge_root = root.find("m:mergeCells", NS)
    if merge_root is None:
        return merged_cells

    for merge_cell in merge_root.findall("m:mergeCell", NS):
        ref = merge_cell.attrib.get("ref", "")
        if ":" not in ref:
            continue
        start_ref, end_ref = ref.split(":", 1)
        start_col, start_row = split_cell_ref(start_ref)
        end_col, end_row = split_cell_ref(end_ref)
        for row in range(start_row, end_row + 1):
            for col in range(start_col, end_col + 1):
                if row == start_row and col == start_col:
                    continue
                merged_cells[(row, col)] = (start_row, start_col)
    return merged_cells


def markdown_table(rows: list[list[str]]) -> list[str]:
    if len(rows) == 1:
        return ["_No content._"]

    col_count = max(len(row) for row in rows)
    normalized_rows = [row + [""] * (col_count - len(row)) for row in rows]
    output = [
        "| " + " | ".join(escape_table_cell(cell) for cell in normalized_rows[0]) + " |",
        "| " + " | ".join("---" for _ in range(col_count)) + " |",
    ]
    for row in normalized_rows[1:]:
        output.append("| " + " | ".join(escape_table_cell(cell) for cell in row) + " |")
    return output


def split_cell_ref(ref: str) -> tuple[int, int]:
    match = re.fullmatch(r"([A-Z]+)(\d+)", ref)
    if not match:
        raise ValueError(f"Unsupported cell reference: {ref}")
    return col_to_num(match.group(1)), int(match.group(2))


def max_from_dimension(dimension: str) -> tuple[int, int]:
    if not dimension:
        return 0, 0
    last = dimension.split(":")[-1]
    col, row = split_cell_ref(last)
    return row, col


def col_to_num(col: str) -> int:
    number = 0
    for char in col:
        number = number * 26 + ord(char.upper()) - ord("A") + 1
    return number


def num_to_col(number: int) -> str:
    col = ""
    while number:
        number, remainder = divmod(number - 1, 26)
        col = chr(ord("A") + remainder) + col
    return col


def normalize_cell_text(value: str) -> str:
    text = value.replace("\r\n", "\n").replace("\r", "\n")
    return "\n".join(line.rstrip() for line in text.split("\n")).strip()


def normalize_call_no(value: str) -> str:
    value = value.strip()
    if not value:
        return ""
    if not re.fullmatch(r"\d+(?:\.0*)?", value):
        return ""
    return f"{int(float(value)):03d}"


def call_anchor(sheet: SheetInfo, call_no: str) -> str:
    return f"{sheet.name}_{call_no}"


def sheet_anchor(sheet: SheetInfo) -> str:
    return f"sheet-{sheet.index}-{safe_anchor(sheet.name)}"


def safe_anchor(value: str) -> str:
    safe = re.sub(r"[^0-9A-Za-z_-]+", "-", value.strip()).strip("-")
    return safe or "item"


def anchor_tag(anchor: str) -> str:
    return f'<a id="{html.escape(anchor, quote=True)}"></a>'


def escape_table_cell(value: str) -> str:
    escaped = escape_md(value)
    return escaped.replace("\n", "<br>").replace("|", "\\|")


def escape_md(value: str) -> str:
    return value.replace("\\", "\\\\")


if __name__ == "__main__":
    main()
