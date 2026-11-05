// ArtCraft Suite — MCA / MCP Command Executor for SheetApp

impl SheetApp {
    fn ensure_active_workbook_for_agent(&mut self) {
        if self.session.active().is_none() {
            self.session.new_workbook();
        }
        self.backstage = None;
    }

    fn col_idx_to_letters(mut col: u32) -> String {
        let mut out = Vec::new();
        loop {
            out.push((b'A' + (col % 26) as u8) as char);
            if col < 26 {
                break;
            }
            col = col / 26 - 1;
        }
        out.into_iter().rev().collect()
    }

    fn parse_a1_coords(cell: &str) -> (u32, u32) {
        let clean = cell.rsplit_once('!').map(|(_, b)| b).unwrap_or(cell).trim();
        let mut col_part = String::new();
        let mut row_part = String::new();
        for ch in clean.chars() {
            if ch.is_ascii_alphabetic() && row_part.is_empty() {
                col_part.push(ch.to_ascii_uppercase());
            } else if ch.is_ascii_digit() {
                row_part.push(ch);
            }
        }
        let col = if col_part.is_empty() {
            0
        } else {
            col_part
                .chars()
                .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1))
                .saturating_sub(1)
        };
        let row = row_part.parse::<u32>().unwrap_or(1).saturating_sub(1);
        (col, row)
    }

    fn create_fancy_table_helper(&mut self, spec: &serde_json::Value) -> Result<serde_json::Value, String> {
        self.ensure_active_workbook_for_agent();
        let start_cell = spec
            .get("startCell")
            .or_else(|| spec.get("range"))
            .or_else(|| spec.get("cell"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("A1");
        let (start_col, mut cur_row) = Self::parse_a1_coords(start_cell);

        let header_bg = spec
            .get("themeColor")
            .or_else(|| spec.get("headerBg"))
            .or_else(|| spec.get("headerColor"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("#1E293B");
        let zebra = spec.get("zebraStripes").and_then(serde_json::Value::as_bool).unwrap_or(true);

        if let Some(title) = spec.get("title").and_then(serde_json::Value::as_str) {
            if !title.trim().is_empty() {
                let title_a1 = format!("{}{}", Self::col_idx_to_letters(start_col), cur_row + 1);
                let _ = self.run("cell.set", serde_json::json!({"cell": title_a1, "input": title.trim()}));
                let _ = self.run("home.bold", serde_json::json!({"range": title_a1, "on": true}));
                let _ = self.run("home.fontSize", serde_json::json!({"range": title_a1, "size": 14}));
                cur_row += 1;
            }
        }

        let headers: Vec<String> = spec
            .get("headers")
            .or_else(|| spec.get("columns"))
            .and_then(serde_json::Value::as_array)
            .map(|arr| {
                arr.iter()
                    .map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string().trim_matches('"').to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let rows: Vec<Vec<serde_json::Value>> = spec
            .get("rows")
            .or_else(|| spec.get("data"))
            .or_else(|| spec.get("values"))
            .and_then(serde_json::Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r.as_array().cloned())
                    .collect()
            })
            .unwrap_or_default();

        let cols = headers
            .len()
            .max(rows.iter().map(Vec::len).max().unwrap_or(0))
            .max(1) as u32;
        let table_start_row = cur_row;

        if !headers.is_empty() {
            for (c_idx, h_text) in headers.iter().enumerate() {
                let cell_a1 = format!("{}{}", Self::col_idx_to_letters(start_col + c_idx as u32), cur_row + 1);
                let clean = h_text.replace("**", "");
                let _ = self.run("cell.set", serde_json::json!({"cell": cell_a1, "input": clean}));
            }
            let hdr_range = format!(
                "{}{}:{}{}",
                Self::col_idx_to_letters(start_col),
                cur_row + 1,
                Self::col_idx_to_letters(start_col + cols - 1),
                cur_row + 1
            );
            let _ = self.run("home.bold", serde_json::json!({"range": hdr_range, "on": true}));
            let bg_hex = if header_bg.starts_with('#') { header_bg.to_string() } else { format!("#{header_bg}") };
            let _ = self.run("home.fillColor", serde_json::json!({"range": hdr_range, "color": bg_hex}));
            let _ = self.run("home.fontColor", serde_json::json!({"range": hdr_range, "color": "#FFFFFF"}));
            cur_row += 1;
        }

        for (r_idx, row_vals) in rows.iter().enumerate() {
            for (c_idx, val) in row_vals.iter().enumerate() {
                let cell_a1 = format!("{}{}", Self::col_idx_to_letters(start_col + c_idx as u32), cur_row + 1);
                let input_val = match val {
                    serde_json::Value::String(s) => serde_json::json!(s.replace("**", "")),
                    other => other.clone(),
                };
                let _ = self.run("cell.set", serde_json::json!({"cell": cell_a1, "input": input_val}));
            }
            if zebra && r_idx % 2 == 1 {
                let row_range = format!(
                    "{}{}:{}{}",
                    Self::col_idx_to_letters(start_col),
                    cur_row + 1,
                    Self::col_idx_to_letters(start_col + cols - 1),
                    cur_row + 1
                );
                let _ = self.run("home.fillColor", serde_json::json!({"range": row_range, "color": "#F1F5F9"}));
            }
            cur_row += 1;
        }

        // Optional formulas (either array of {cell, formula} or object map {"B10": "=SUM(B2:B9)"})
        if let Some(f_arr) = spec.get("formulas").and_then(serde_json::Value::as_array) {
            for item in f_arr {
                if let (Some(c), Some(f)) = (
                    item.get("cell").and_then(serde_json::Value::as_str),
                    item.get("formula").or_else(|| item.get("value")).and_then(serde_json::Value::as_str),
                ) {
                    let formula_str = if f.starts_with('=') { f.to_string() } else { format!("={f}") };
                    let _ = self.run("cell.set", serde_json::json!({"cell": c, "input": formula_str}));
                    let _ = self.run("home.bold", serde_json::json!({"range": c, "on": true}));
                }
            }
        } else if let Some(f_obj) = spec.get("formulas").and_then(serde_json::Value::as_object) {
            for (c, f_val) in f_obj {
                if let Some(f) = f_val.as_str() {
                    let formula_str = if f.starts_with('=') { f.to_string() } else { format!("={f}") };
                    let _ = self.run("cell.set", serde_json::json!({"cell": c, "input": formula_str}));
                    let _ = self.run("home.bold", serde_json::json!({"range": c, "on": true}));
                }
            }
        }

        let end_row = cur_row.max(table_start_row + 1);
        let full_range = format!(
            "{}{}:{}{}",
            Self::col_idx_to_letters(start_col),
            table_start_row + 1,
            Self::col_idx_to_letters(start_col + cols - 1),
            end_row
        );
        let _ = self.run("home.borders", serde_json::json!({"range": full_range, "preset": "all", "color": "#CBD5E1"}));

        Ok(serde_json::json!({
            "status": "ok",
            "range": full_range,
            "rows_written": rows.len(),
            "cols": cols
        }))
    }

    fn apply_format_range_helper(&mut self, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        self.ensure_active_workbook_for_agent();
        let range = params.get("range").and_then(serde_json::Value::as_str).unwrap_or("A1");
        if let Some(b) = params.get("bold").and_then(serde_json::Value::as_bool) {
            let _ = self.run("home.bold", serde_json::json!({"range": range, "on": b}));
        }
        if let Some(it) = params.get("italic").and_then(serde_json::Value::as_bool) {
            let _ = self.run("home.italic", serde_json::json!({"range": range, "on": it}));
        }
        if let Some(sz) = params.get("fontSize").or_else(|| params.get("size")).and_then(serde_json::Value::as_f64) {
            let _ = self.run("home.fontSize", serde_json::json!({"range": range, "size": sz}));
        }
        if let Some(fc) = params.get("fontColor").or_else(|| params.get("color")).and_then(serde_json::Value::as_str) {
            let hex = if fc.starts_with('#') || fc == "auto" { fc.to_string() } else { format!("#{fc}") };
            let _ = self.run("home.fontColor", serde_json::json!({"range": range, "color": hex}));
        }
        if let Some(bg) = params.get("fillColor").or_else(|| params.get("background")).and_then(serde_json::Value::as_str) {
            let hex = if bg.starts_with('#') || bg == "none" { bg.to_string() } else { format!("#{bg}") };
            let _ = self.run("home.fillColor", serde_json::json!({"range": range, "color": hex}));
        }
        if let Some(nf) = params.get("numberFormat").or_else(|| params.get("format")).and_then(serde_json::Value::as_str) {
            let _ = self.run("home.numberFormat", serde_json::json!({"range": range, "format": nf}));
        }
        Ok(serde_json::json!({"status": "ok", "range": range}))
    }

    /// Polls and executes AI Agent tool calls on the main thread for GridCraft.
    pub fn poll_artcraft_agent(&mut self) {
        let reqs = self.artcraft_agent.take_tool_requests();
        for req in reqs {
            let res = match req.tool_name.as_str() {
                "grid.createFancyTable" | "create_fancy_table" | "grid.populateRange" => {
                    self.create_fancy_table_helper(&req.params)
                }
                "insert_table_from_chat" => {
                    let md = req.params.get("markdown").or_else(|| req.params.get("text")).and_then(serde_json::Value::as_str).unwrap_or("");
                    let mut parsed_rows: Vec<Vec<serde_json::Value>> = Vec::new();
                    let mut title_opt: Option<String> = None;
                    for line in md.lines() {
                        let t = line.trim();
                        if t.starts_with("# ") || t.starts_with("## ") {
                            if title_opt.is_none() {
                                title_opt = Some(t.trim_start_matches('#').trim().replace("**", ""));
                            }
                        } else if t.starts_with('|') && t.ends_with('|') && t.len() > 2 {
                            let inner = &t[1..t.len() - 1];
                            let cells: Vec<String> = inner.split('|').map(|c| c.trim().replace("**", "")).collect();
                            let is_sep = cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')));
                            if !is_sep {
                                parsed_rows.push(cells.into_iter().map(serde_json::Value::String).collect());
                            }
                        } else if let Some(b) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
                            if let Some((k, v)) = b.split_once(':') {
                                parsed_rows.push(vec![
                                    serde_json::Value::String(k.trim().replace("**", "")),
                                    serde_json::Value::String(v.trim().replace("**", "")),
                                ]);
                            }
                        }
                    }
                    if !parsed_rows.is_empty() {
                        let headers: Vec<String> = parsed_rows
                            .remove(0)
                            .into_iter()
                            .map(|v| v.as_str().unwrap_or("").to_string())
                            .collect();
                        self.create_fancy_table_helper(&serde_json::json!({
                            "startCell": "A1",
                            "title": title_opt,
                            "headers": headers,
                            "rows": parsed_rows,
                            "zebraStripes": true
                        }))
                    } else {
                        self.ensure_active_workbook_for_agent();
                        let _ = self.run("cell.set", serde_json::json!({"cell": "A1", "input": md.lines().next().unwrap_or("")}));
                        Ok(serde_json::json!({"status": "ok"}))
                    }
                }
                "write_range" | "range.setValues" => {
                    self.ensure_active_workbook_for_agent();
                    let range = req.params.get("range").or_else(|| req.params.get("startCell")).and_then(serde_json::Value::as_str).unwrap_or("A1");
                    let values = req.params.get("values").or_else(|| req.params.get("rows")).cloned().unwrap_or_else(|| serde_json::json!([]));
                    self.run("range.setValues", serde_json::json!({"range": range, "values": values})).map_err(|e| e.to_string())
                }
                "format_range" | "grid.formatRange" => {
                    self.apply_format_range_helper(&req.params)
                }
                "insert_chart" | "grid.insertChart" | "insert.chart" => {
                    self.ensure_active_workbook_for_agent();
                    self.run("insert.chart", req.params.clone()).map_err(|e| e.to_string())
                }
                "set_cell_value" | "set_cell" | "cell.set" => {
                    self.ensure_active_workbook_for_agent();
                    let cell_a1 = if let Some(c) = req.params.get("cell").and_then(serde_json::Value::as_str) {
                        c.to_string()
                    } else {
                        let col = req.params.get("col").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
                        let row = req.params.get("row").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
                        format!("{}{}", Self::col_idx_to_letters(col), row + 1)
                    };
                    let val = req
                        .params
                        .get("input")
                        .or_else(|| req.params.get("value"))
                        .or_else(|| req.params.get("text"))
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!(""));
                    self.run("cell.set", serde_json::json!({"cell": cell_a1, "input": val})).map_err(|e| e.to_string())
                }
                "execute" => {
                    self.ensure_active_workbook_for_agent();
                    let cmd = req.params.get("command").and_then(serde_json::Value::as_str).unwrap_or("");
                    let params = req.params.get("params").cloned().unwrap_or(serde_json::json!({}));
                    match cmd {
                        "grid.createFancyTable" | "create_fancy_table" | "grid.populateRange" => self.create_fancy_table_helper(&params),
                        "format_range" | "grid.formatRange" => self.apply_format_range_helper(&params),
                        "write_range" => {
                            let range = params.get("range").and_then(serde_json::Value::as_str).unwrap_or("A1");
                            let values = params.get("values").or_else(|| params.get("rows")).cloned().unwrap_or_else(|| serde_json::json!([]));
                            self.run("range.setValues", serde_json::json!({"range": range, "values": values})).map_err(|e| e.to_string())
                        }
                        "set_cell" | "set_cell_value" | "cell.setText" => {
                            let cell_a1 = if let Some(c) = params.get("cell").and_then(serde_json::Value::as_str) {
                                c.to_string()
                            } else {
                                let col = params.get("col").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
                                let row = params.get("row").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
                                format!("{}{}", Self::col_idx_to_letters(col), row + 1)
                            };
                            let val = params
                                .get("input")
                                .or_else(|| params.get("value"))
                                .or_else(|| params.get("text"))
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!(""));
                            self.run("cell.set", serde_json::json!({"cell": cell_a1, "input": val})).map_err(|e| e.to_string())
                        }
                        other_cmd => self.run(other_cmd, params).map_err(|e| e.to_string()),
                    }
                }
                "inspect_workbook" | "inspect_document" | "get_text" => {
                    let title = self.session.active().map(|d| d.display_title()).unwrap_or_else(|| "Unbenannt".into());
                    let dirty = self.session.active().map(|d| d.is_dirty()).unwrap_or(false);
                    let sheets: Vec<String> = self.session.active().map(|d| d.wb.sheets.iter().map(|s| s.name.clone()).collect()).unwrap_or_default();
                    let path = self.session.active().and_then(|d| d.path.clone());
                    Ok(serde_json::json!({
                        "title": title,
                        "dirty": dirty,
                        "sheets": sheets,
                        "path": path,
                    }))
                }
                "save_document" => {
                    let target = if let Some(p) = req.params.get("path").and_then(serde_json::Value::as_str) {
                        std::path::PathBuf::from(p)
                    } else if let Some(ref path) = self.session.active().and_then(|d| d.path.clone()) {
                        std::path::PathBuf::from(path)
                    } else {
                        self.artcraft_drive.set_or_init_project(None)
                    };
                    let r = self.run("file.saveAs", serde_json::json!({"path": target.to_string_lossy().to_string()})).map_err(|e| e.to_string());
                    if r.is_ok() {
                        self.artcraft_drive.mark_saved();
                    }
                    r
                }
                other => {
                    self.ensure_active_workbook_for_agent();
                    self.run(other, req.params.clone()).map_err(|e| e.to_string())
                }
            };
            let _ = req.reply_sender.send(res);
        }

        self.artcraft_agent.process_events();
    }

    /// Automatically saves changes into the active Google Drive project folder if enabled.
    pub fn artcraft_autosave_tick(&mut self) {
        let is_dirty = self.session.active().map(|d| d.is_dirty()).unwrap_or(false);
        if self.artcraft_drive.auto_save_enabled && is_dirty && self.artcraft_drive.should_autosave() {
            let target = if let Some(ref path) = self.session.active().and_then(|d| d.path.clone()) {
                std::path::PathBuf::from(path)
            } else {
                self.artcraft_drive.set_or_init_project(None)
            };
            let target_str = target.to_string_lossy().to_string();
            if self.run("file.saveAs", serde_json::json!({"path": target_str})).is_ok() {
                self.artcraft_drive.mark_saved();
                self.toast = Some((format!("☁️ Gespeichert in {}", self.artcraft_drive.active_project_folder_name()), now_ms()));
            }
        }
    }
}
