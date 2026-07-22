use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::sync::{SyncDailyReportRequest, SyncResult};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

const DAILY_INPUT_URL: &str = "http://192.168.10.89/dailyinput";

pub async fn sync_daily_report(
    app: tauri::AppHandle,
    request: SyncDailyReportRequest,
) -> AppResult<SyncResult> {
    let total_count = request.entries.len();
    if total_count == 0 {
        return Ok(SyncResult {
            success: false,
            message: "Không có dữ liệu để đồng bộ.".to_string(),
            synced_count: 0,
            total_count: 0,
        });
    }

    let entries_json = serde_json::to_string(&request.entries)
        .map_err(|e| AppError::new(format!("Lỗi serialize dữ liệu: {}", e)))?;

    let init_script = build_init_script(&entries_json, &request.date);

    if let Some(existing) = app.get_webview_window("sync-browser") {
        let _ = existing.destroy();
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    let url = DAILY_INPUT_URL
        .parse::<tauri::Url>()
        .map_err(|_| AppError::new("URL không hợp lệ."))?;

    WebviewWindowBuilder::new(&app, "sync-browser", WebviewUrl::External(url))
        .title(format!("Đồng bộ hệ thống nội bộ — {}", request.date))
        .maximized(true)
        .min_inner_size(800.0, 500.0)
        .initialization_script(&init_script)
        .build()
        .map_err(|e| AppError::new(format!("Không thể mở cửa sổ trình duyệt: {}", e)))?;

    Ok(SyncResult {
        success: true,
        message: format!(
            "Đã mở cửa sổ đồng bộ. Ngày: {}. Đăng nhập xong hệ thống sẽ tự động điền dữ liệu.",
            request.date
        ),
        synced_count: 0,
        total_count,
    })
}

fn build_init_script(entries_json: &str, date: &str) -> String {
    format!(
        r#"
(function() {{
  const SYNC_DATA = {{
    date: "{date}",
    entries: {entries_json},
  }};

  if (window.__SYNC_FILLED__) return;

  const FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif";

  /* ── Overlay chặn thao tác ── */
  function createOverlay() {{
    if (document.getElementById('sync-overlay')) return;

    const overlay = document.createElement('div');
    overlay.id = 'sync-overlay';
    overlay.style.cssText = `
      position: fixed; inset: 0; z-index: 999990;
      background: rgba(0,0,0,0.35);
      pointer-events: auto;
      cursor: not-allowed;
    `;
    overlay.addEventListener('click', e => {{ e.stopPropagation(); e.preventDefault(); }}, true);
    overlay.addEventListener('mousedown', e => {{ e.stopPropagation(); e.preventDefault(); }}, true);
    overlay.addEventListener('keydown', e => {{ e.stopPropagation(); e.preventDefault(); }}, true);
    document.body.appendChild(overlay);

    const style = document.createElement('style');
    style.id = 'sync-block-style';
    style.textContent = `
      input:not(#sync-progress *), select:not(#sync-progress *), textarea:not(#sync-progress *),
      button:not(#sync-progress *), a:not(#sync-progress *) {{
        pointer-events: none !important;
      }}
    `;
    document.head.appendChild(style);

    lockFormFields(true);
  }}

  function lockFormFields(lock) {{
    const doc = getFormDoc();
    const fields = doc.querySelectorAll('input, select, textarea, button');
    fields.forEach(f => {{
      if (lock) {{
        f.dataset.syncPrevTabindex = f.getAttribute('tabindex') || '';
        f.setAttribute('tabindex', '-1');
        f.style.pointerEvents = 'none';
      }} else {{
        const prev = f.dataset.syncPrevTabindex;
        if (prev === '') f.removeAttribute('tabindex');
        else if (prev) f.setAttribute('tabindex', prev);
        delete f.dataset.syncPrevTabindex;
        f.style.pointerEvents = '';
      }}
    }});
  }}

  function removeOverlay() {{
    const el = document.getElementById('sync-overlay');
    if (el) el.remove();
    const style = document.getElementById('sync-block-style');
    if (style) style.remove();
    lockFormFields(false);
  }}

  /* ── Progress panel (hiển thị trên overlay) ── */
  function createProgressPanel() {{
    if (document.getElementById('sync-progress')) return;

    const panel = document.createElement('div');
    panel.id = 'sync-progress';
    panel.style.cssText = `
      position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
      z-index: 999999; background: white; border-radius: 12px;
      box-shadow: 0 8px 32px rgba(0,0,0,0.25); width: 520px; max-height: 80vh;
      overflow: auto; font-family: ${{FONT}};
    `;

    /* Header */
    const header = document.createElement('div');
    header.style.cssText = `
      padding: 16px 20px; background: linear-gradient(135deg, #059669, #047857);
      border-radius: 12px 12px 0 0; color: white;
    `;
    header.innerHTML = '<div style="font-weight:700;font-size:15px">Đang đồng bộ dữ liệu</div>' +
      '<div style="font-size:12px;opacity:0.85;margin-top:4px">Ngày: ' + SYNC_DATA.date +
      ' — ' + SYNC_DATA.entries.length + ' mục — Tổng: ' +
      SYNC_DATA.entries.reduce((s, e) => s + e.hour, 0).toFixed(1) + 'h</div>';

    /* Progress bar */
    const progressWrap = document.createElement('div');
    progressWrap.style.cssText = 'padding: 12px 20px; border-bottom: 1px solid #e5e7eb;';
    progressWrap.innerHTML = `
      <div style="display:flex;align-items:center;gap:10px">
        <div style="flex:1;height:8px;background:#e5e7eb;border-radius:4px;overflow:hidden">
          <div id="sync-bar" style="height:100%;width:0%;background:#059669;border-radius:4px;transition:width 0.4s"></div>
        </div>
        <span id="sync-pct" style="font-size:12px;font-weight:700;color:#059669;min-width:42px;text-align:right">0%</span>
      </div>
      <div id="sync-step" style="font-size:12px;color:#6b7280;margin-top:6px">Đang chờ trang tải...</div>
    `;

    /* Log */
    const logWrap = document.createElement('div');
    logWrap.id = 'sync-log';
    logWrap.style.cssText = 'padding: 8px 20px 16px; max-height: 320px; overflow: auto;';

    panel.append(header, progressWrap, logWrap);
    document.body.appendChild(panel);
  }}

  function setProgress(pct, stepText) {{
    const bar = document.getElementById('sync-bar');
    const pctEl = document.getElementById('sync-pct');
    const step = document.getElementById('sync-step');
    if (bar) bar.style.width = pct + '%';
    if (pctEl) pctEl.textContent = Math.round(pct) + '%';
    if (step) step.textContent = stepText;
  }}

  function addLog(icon, text, color) {{
    const log = document.getElementById('sync-log');
    if (!log) return;
    const row = document.createElement('div');
    row.style.cssText = 'padding:4px 0;font-size:12px;color:' + (color || '#374151') + ';display:flex;align-items:center;gap:6px;';
    row.innerHTML = '<span>' + icon + '</span><span>' + text + '</span>';
    log.appendChild(row);
    log.scrollTop = log.scrollHeight;
  }}

  /* ── Dialog xác nhận cập nhật ── */
  function showUpdateDialog() {{
    if (document.getElementById('sync-update-dialog')) return;

    const backdrop = document.createElement('div');
    backdrop.id = 'sync-update-dialog';
    backdrop.style.cssText = `
      position: fixed; inset: 0; z-index: 999999;
      background: rgba(0,0,0,0.4);
      display: flex; align-items: center; justify-content: center;
    `;

    const dialog = document.createElement('div');
    dialog.style.cssText = `
      background: white; border-radius: 10px; padding: 24px 28px;
      box-shadow: 0 8px 32px rgba(0,0,0,0.25); min-width: 340px;
      font-family: ${{FONT}};
    `;

    const title = document.createElement('div');
    title.style.cssText = 'font-size: 16px; font-weight: 700; color: #1f2937; margin-bottom: 20px;';
    title.textContent = 'Cập nhật dữ liệu';

    const btnRow = document.createElement('div');
    btnRow.style.cssText = 'display: flex; justify-content: flex-end; gap: 10px;';

    const btnCancel = document.createElement('button');
    btnCancel.textContent = 'Cancel';
    btnCancel.style.cssText = `
      padding: 8px 20px; border-radius: 6px; font-size: 14px; font-weight: 600;
      border: 1px solid #d1d5db; background: white; color: #374151; cursor: pointer;
    `;
    btnCancel.onmouseover = () => {{ btnCancel.style.background = '#f3f4f6'; }};
    btnCancel.onmouseout = () => {{ btnCancel.style.background = 'white'; }};
    btnCancel.onclick = () => {{ backdrop.remove(); }};

    const btnOk = document.createElement('button');
    btnOk.textContent = 'OK';
    btnOk.style.cssText = `
      padding: 8px 20px; border-radius: 6px; font-size: 14px; font-weight: 600;
      border: none; background: #059669; color: white; cursor: pointer;
    `;
    btnOk.onmouseover = () => {{ btnOk.style.background = '#047857'; }};
    btnOk.onmouseout = () => {{ btnOk.style.background = '#059669'; }};
    btnOk.onclick = () => {{
      backdrop.remove();
      const doc = getFormDoc();
      const btn = doc.getElementById('btnSaveDisplay') || document.getElementById('btnSaveDisplay');
      if (btn) {{
        btn.style.pointerEvents = '';
        btn.removeAttribute('tabindex');
        btn.click();
      }}
    }};

    btnRow.append(btnCancel, btnOk);
    dialog.append(title, btnRow);
    backdrop.appendChild(dialog);
    document.body.appendChild(backdrop);
  }}

  /* ── Điền dữ liệu tự động ── */
  function sleep(ms) {{
    return new Promise(r => setTimeout(r, ms));
  }}

  function isLoginPage() {{
    const url = window.location.href.toLowerCase();
    if (url.includes('login') || url.includes('signin') || url.includes('auth')) return true;
    if (document.querySelector("input[type='password']") && !url.includes('dailyinput')) return true;
    return false;
  }}

  function removeProgressPanel() {{
    const el = document.getElementById('sync-progress');
    if (el) el.remove();
  }}

  const DEFAULT_ROWS = 3;

  /* ── Tìm document chứa form (hỗ trợ iframe) ── */
  function getFormDoc() {{
    const iframes = document.querySelectorAll('iframe');
    for (const iframe of iframes) {{
      try {{
        const doc = iframe.contentDocument || iframe.contentWindow.document;
        if (doc && doc.querySelector('input, select, textarea')) return doc;
      }} catch(e) {{}}
    }}
    return document;
  }}

  function findField(doc, name) {{
    return doc.querySelector('[name="' + name + '"]')
        || doc.getElementById(name)
        || doc.querySelector('[id="' + name + '"]');
  }}

  function unlockField(el) {{
    el.style.pointerEvents = '';
    el.removeAttribute('tabindex');
  }}

  function relockField(el) {{
    el.style.pointerEvents = 'none';
    el.setAttribute('tabindex', '-1');
  }}

  function setFieldValue(doc, name, value) {{
    const el = findField(doc, name);
    if (!el) return false;
    unlockField(el);
    el.focus();
    el.value = String(value);
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
    el.dispatchEvent(new Event('blur', {{ bubbles: true }}));
    relockField(el);
    return true;
  }}

  function setSelectValue(doc, name, value) {{
    const el = findField(doc, name);
    if (!el || !el.options) return false;
    unlockField(el);
    for (let i = 0; i < el.options.length; i++) {{
      if (el.options[i].value === value || el.options[i].text === value) {{
        el.selectedIndex = i;
        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
        relockField(el);
        return true;
      }}
    }}
    el.value = value;
    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
    relockField(el);
    return true;
  }}

  function clickAddButton(doc) {{
    const btns = doc.querySelectorAll('input[type="button"], button, input[type="submit"], a');
    for (const btn of btns) {{
      const text = (btn.value || btn.textContent || btn.innerText || '').toLowerCase();
      if (text.includes('add') || text.includes('追加') || text.includes('thêm') || text.includes('行追加')) {{
        btn.click();
        return true;
      }}
    }}
    return false;
  }}

  function countExistingRows(doc) {{
    let count = 0;
    while (findField(doc, 'txtProjectCD' + (count + 1))) {{
      count++;
    }}
    return count;
  }}

  function scanFields(doc) {{
    const fields = doc.querySelectorAll('input, select, textarea');
    const names = [];
    fields.forEach(f => {{
      const n = f.name || f.id || '';
      if (n) names.push(n + ' (' + f.tagName.toLowerCase() + ')');
    }});
    return names;
  }}

  async function ensureRows(doc, needed) {{
    let current = countExistingRows(doc);
    if (current === 0) current = DEFAULT_ROWS;
    // addLog('📊', 'Số dòng hiện tại: ' + current + ', cần: ' + needed, '#6b7280');
    let added = 0;
    while (current < needed) {{
      const ok = clickAddButton(doc);
      if (!ok) {{
        addLog('⚠️', 'Không tìm thấy nút Add để thêm dòng.', '#dc2626');
        break;
      }}
      await sleep(500);
      current++;
      added++;
      addLog('➕', 'Đã thêm dòng ' + current, '#6b7280');
    }}
    if (added > 0) {{
      addLog('✅', 'Đã thêm ' + added + ' dòng.', '#059669');
    }}
  }}

  async function autoFill() {{
    if (isLoginPage()) {{
      removeOverlay();
      removeProgressPanel();
      return;
    }}

    createOverlay();
    createProgressPanel();

    const total = SYNC_DATA.entries.length;

    setProgress(2, 'Đang quét trang...');
    // addLog('🔍', 'URL: ' + window.location.href, '#6b7280');
    await sleep(500);

    const doc = getFormDoc();
    const isIframe = doc !== document;
    // addLog('📄', isIframe ? 'Form nằm trong iframe.' : 'Form nằm trên trang chính.', '#6b7280');

    const fieldNames = scanFields(doc);
    if (fieldNames.length === 0) {{
      addLog('❌', 'Không tìm thấy field nào trên trang. Trang chưa tải xong?', '#dc2626');
      setProgress(100, 'Lỗi — không tìm thấy form.');
      await sleep(2000);
      removeOverlay();
      return;
    }}

    // addLog('📋', 'Tìm thấy ' + fieldNames.length + ' fields. Mẫu: ' + fieldNames.slice(0, 8).join(', '), '#6b7280');

    const testField = findField(doc, 'txtProjectCD1');
    if (!testField) {{
      addLog('⚠️', 'Không tìm thấy txtProjectCD1. Danh sách field:', '#dc2626');
      fieldNames.slice(0, 20).forEach(n => addLog('  ', n, '#9ca3af'));
      setProgress(100, 'Lỗi — field name không khớp.');
      await sleep(2000);
      removeOverlay();
      return;
    }}

    setProgress(5, 'Đang kiểm tra số dòng...');
    await ensureRows(doc, total);
    await sleep(300);

    let filled = 0;
    for (let i = 0; i < total; i++) {{
      const entry = SYNC_DATA.entries[i];
      const idx = i + 1;
      const pct = Math.round(5 + ((i + 1) / total) * 95);
      const label = entry.project_code + ' — ' + (entry.category_label ? '【' + entry.category_label + '】' : '') + entry.task_name;

      setProgress(pct, 'Đang điền dòng ' + idx + '/' + total);

      const r1 = setFieldValue(doc, 'txtProjectCD' + idx, entry.project_code);
      const r2 = setSelectValue(doc, 'cmbProcess' + idx, entry.process_code);
      const r3 = setFieldValue(doc, 'txtWorkHour' + idx, entry.hour);
      const r4 = setFieldValue(doc, 'txtWorkDetail' + idx, entry.task_name);

      let otInfo = '';
      if (entry.regular_ot > 0) {{
        setFieldValue(doc, 'txtNormalOTHour' + idx, entry.regular_ot);
        otInfo += ' OT:' + entry.regular_ot;
      }}
      if (entry.midnight_ot > 0) {{
        setFieldValue(doc, 'txtMidnightOTHour' + idx, entry.midnight_ot);
        otInfo += ' 深夜:' + entry.midnight_ot;
      }}

      const status = [
        r1 ? 'PJ✓' : 'PJ✗',
        r2 ? 'Process✓' : 'Process✗',
        r3 ? 'Hour✓' : 'Hour✗',
        r4 ? 'Detail✓' : 'Detail✗',
      ].join(' ');

      if (r1 && r3) filled++;
      const color = (r1 && r3) ? '#374151' : '#dc2626';
      // addLog('✏️', 'Dòng ' + idx + ': ' + label + ' → ' + entry.hour + 'h' + otInfo + '  [' + status + ']', color);
      await sleep(300);
    }}

    window.__SYNC_FILLED__ = true;
    setProgress(100, 'Hoàn tất — ' + filled + '/' + total + ' mục đã điền.');
    addLog('✅', 'Đã điền xong ' + filled + '/' + total + ' mục.', '#059669');

    await sleep(1500);
    removeProgressPanel();
    removeOverlay();
    showUpdateDialog();
  }}

  /* ── Chờ form xuất hiện rồi mới fill ── */
  function waitForFormAndFill() {{
    if (isLoginPage()) {{
      removeOverlay();
      removeProgressPanel();
      return;
    }}

    const doc = getFormDoc();
    const hasField = findField(doc, 'txtProjectCD1')
                  || doc.querySelectorAll('input, select').length > 5;

    if (hasField) {{
      autoFill();
      return;
    }}

    let retries = 0;
    const maxRetries = 30;
    const observer = new MutationObserver(() => {{
      retries++;
      const d = getFormDoc();
      const found = findField(d, 'txtProjectCD1')
                 || d.querySelectorAll('input, select').length > 5;
      if (found || retries >= maxRetries) {{
        observer.disconnect();
        autoFill();
      }}
    }});
    observer.observe(document.body || document.documentElement, {{ childList: true, subtree: true }});

    setTimeout(() => {{
      observer.disconnect();
      autoFill();
    }}, 10000);
  }}

  function start() {{
    setTimeout(waitForFormAndFill, 800);
  }}

  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', start);
  }} else {{
    start();
  }}
}})();
"#,
        date = date,
        entries_json = entries_json,
    )
}
