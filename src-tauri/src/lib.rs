use chrono::Utc;
use rusqlite::{params, Connection, Transaction};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const SCHEMA_VERSION: i64 = 2;

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("micuadernodigital.sqlite3"))
}

fn open_db(app: &AppHandle) -> Result<Connection, String> {
    let path = db_path(app)?;
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| e.to_string())?;
    conn.execute_batch(r#"
      CREATE TABLE IF NOT EXISTS schema_meta(key TEXT PRIMARY KEY,value TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS app_state(id INTEGER PRIMARY KEY CHECK(id=1),state_json TEXT NOT NULL,updated_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS state_backups(id INTEGER PRIMARY KEY AUTOINCREMENT,state_json TEXT NOT NULL,created_at TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_state_backups_created_at ON state_backups(created_at);

      CREATE TABLE IF NOT EXISTS students(id TEXT PRIMARY KEY,name TEXT NOT NULL,status TEXT,attendance REAL,feedbacks INTEGER,coverage REAL,strengths_json TEXT,difficulties_json TEXT,next_step TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS observations(id TEXT PRIMARY KEY,date TEXT,student_id TEXT,category TEXT,text TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_observations_student_date ON observations(student_id,date);
      CREATE TABLE IF NOT EXISTS attendance_entries(date TEXT NOT NULL,student_id TEXT NOT NULL,status TEXT NOT NULL,PRIMARY KEY(date,student_id));

      CREATE TABLE IF NOT EXISTS units(id TEXT PRIMARY KEY,title TEXT NOT NULL,areas TEXT,challenge TEXT,product TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS sessions(id TEXT PRIMARY KEY,unit_id TEXT NOT NULL,title TEXT,subject TEXT,date TEXT,start_time TEXT,end_time TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_sessions_date_time ON sessions(date,start_time);
      CREATE TABLE IF NOT EXISTS criteria(id TEXT PRIMARY KEY,unit_id TEXT NOT NULL,code TEXT,text TEXT,specific_competence TEXT,ce_text TEXT,descriptors TEXT,source TEXT,grade TEXT,area TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS products(id TEXT PRIMARY KEY,unit_id TEXT NOT NULL,name TEXT NOT NULL,area TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS product_criteria(product_id TEXT NOT NULL,criterion_id TEXT NOT NULL,PRIMARY KEY(product_id,criterion_id));

      CREATE TABLE IF NOT EXISTS evidence_columns(id TEXT PRIMARY KEY,label TEXT NOT NULL,criterion TEXT,criterion_text TEXT,specific_competence TEXT,descriptors TEXT,competency TEXT,unit_id TEXT,product_id TEXT,source TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS grades(student_id TEXT NOT NULL,evidence_id TEXT NOT NULL,grade TEXT NOT NULL,PRIMARY KEY(student_id,evidence_id));
      CREATE INDEX IF NOT EXISTS idx_grades_evidence ON grades(evidence_id);

      CREATE TABLE IF NOT EXISTS agenda(id TEXT PRIMARY KEY,date TEXT,time TEXT,type TEXT,priority TEXT,text TEXT,done INTEGER NOT NULL DEFAULT 0,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_agenda_date ON agenda(date,time);
      CREATE TABLE IF NOT EXISTS meetings(id TEXT PRIMARY KEY,date TEXT,student_id TEXT,type TEXT,start_time TEXT,end_time TEXT,title TEXT,development TEXT,agreements TEXT,followup TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_meetings_date ON meetings(date,start_time);
      CREATE TABLE IF NOT EXISTS meeting_participants(meeting_id TEXT NOT NULL,name TEXT NOT NULL,status TEXT,PRIMARY KEY(meeting_id,name));
      CREATE TABLE IF NOT EXISTS meeting_people(name TEXT PRIMARY KEY);
      CREATE TABLE IF NOT EXISTS custom_days(id TEXT PRIMARY KEY,start TEXT NOT NULL,end TEXT NOT NULL,type TEXT,label TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS issues(id TEXT PRIMARY KEY,created_at TEXT,updated_at TEXT,version TEXT,type TEXT,section TEXT,priority TEXT,title TEXT,description TEXT,expected TEXT,status TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_issues_status_created ON issues(status,created_at);
      CREATE TABLE IF NOT EXISTS settings(id INTEGER PRIMARY KEY CHECK(id=1),settings_json TEXT NOT NULL);
    "#).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO schema_meta(key,value) VALUES('schema_version',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![SCHEMA_VERSION.to_string()]).map_err(|e| e.to_string())?;
    Ok(conn)
}

fn sval(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}
fn json(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".to_string())
}
fn arr<'a>(root: &'a Value, key: &str) -> Vec<&'a Value> {
    root.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn sync_normalized(tx: &Transaction<'_>, root: &Value) -> Result<(), String> {
    for table in [
        "students",
        "observations",
        "attendance_entries",
        "units",
        "sessions",
        "criteria",
        "products",
        "product_criteria",
        "evidence_columns",
        "grades",
        "agenda",
        "meetings",
        "meeting_participants",
        "meeting_people",
        "custom_days",
        "issues",
        "settings",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|e| e.to_string())?;
    }
    for v in arr(root, "students") {
        tx.execute("INSERT INTO students(id,name,status,attendance,feedbacks,coverage,strengths_json,difficulties_json,next_step,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![sval(v,"id"),sval(v,"name"),sval(v,"status"),v.get("attendance").and_then(Value::as_f64),v.get("feedbacks").and_then(Value::as_i64),v.get("coverage").and_then(Value::as_f64),json(v.get("strengths").unwrap_or(&Value::Null)),json(v.get("difficulties").unwrap_or(&Value::Null)),sval(v,"next"),json(v)]).map_err(|e|e.to_string())?;
    }
    for v in arr(root, "observations") {
        tx.execute("INSERT INTO observations(id,date,student_id,category,text,raw_json) VALUES(?1,?2,?3,?4,?5,?6)",params![sval(v,"id"),sval(v,"date"),sval(v,"studentId"),sval(v,"category"),sval(v,"text"),json(v)]).map_err(|e|e.to_string())?;
    }
    if let Some(map) = root.get("attendance").and_then(Value::as_object) {
        for (date, rec) in map {
            if let Some(students) = rec.as_object() {
                for (sid, status) in students {
                    if let Some(st) = status.as_str() {
                        tx.execute("INSERT INTO attendance_entries(date,student_id,status) VALUES(?1,?2,?3)",params![date,sid,st]).map_err(|e|e.to_string())?;
                    }
                }
            }
        }
    }
    for u in arr(root, "units") {
        let uid = sval(u, "id");
        tx.execute("INSERT INTO units(id,title,areas,challenge,product,raw_json) VALUES(?1,?2,?3,?4,?5,?6)",params![uid,sval(u,"title"),sval(u,"areas"),sval(u,"challenge"),sval(u,"product"),json(u)]).map_err(|e|e.to_string())?;
        if let Some(a) = u.get("sessions").and_then(Value::as_array) {
            for v in a {
                tx.execute("INSERT INTO sessions(id,unit_id,title,subject,date,start_time,end_time,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![sval(v,"id"),uid,sval(v,"title"),sval(v,"subject"),sval(v,"date"),sval(v,"startTime"),sval(v,"endTime"),json(v)]).map_err(|e|e.to_string())?;
            }
        }
        if let Some(a) = u.get("curricularCriteria").and_then(Value::as_array) {
            for v in a {
                tx.execute("INSERT INTO criteria(id,unit_id,code,text,specific_competence,ce_text,descriptors,source,grade,area,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![sval(v,"id"),uid,sval(v,"code"),sval(v,"text"),sval(v,"ce"),sval(v,"ceText"),sval(v,"descriptors"),sval(v,"source"),sval(v,"grade"),sval(v,"area"),json(v)]).map_err(|e|e.to_string())?;
            }
        }
        if let Some(a) = u.get("products").and_then(Value::as_array) {
            for v in a {
                let pid = sval(v, "id");
                tx.execute(
                    "INSERT INTO products(id,unit_id,name,area,raw_json) VALUES(?1,?2,?3,?4,?5)",
                    params![pid, uid, sval(v, "name"), sval(v, "area"), json(v)],
                )
                .map_err(|e| e.to_string())?;
                if let Some(ids) = v.get("criterionIds").and_then(Value::as_array) {
                    for cid in ids.iter().filter_map(Value::as_str) {
                        tx.execute("INSERT OR IGNORE INTO product_criteria(product_id,criterion_id) VALUES(?1,?2)",params![pid,cid]).map_err(|e|e.to_string())?;
                    }
                }
            }
        }
    }
    for v in arr(root, "evidenceColumns") {
        tx.execute("INSERT INTO evidence_columns(id,label,criterion,criterion_text,specific_competence,descriptors,competency,unit_id,product_id,source,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![sval(v,"id"),sval(v,"label"),sval(v,"criterion"),sval(v,"criterionText"),sval(v,"specificCompetence"),sval(v,"descriptors"),sval(v,"competency"),sval(v,"unitId"),sval(v,"productId"),sval(v,"source"),json(v)]).map_err(|e|e.to_string())?;
    }
    if let Some(map) = root.get("evidence").and_then(Value::as_object) {
        for (sid, rec) in map {
            if let Some(cols) = rec.as_object() {
                for (eid, g) in cols {
                    if let Some(grade) = g.as_str() {
                        tx.execute(
                            "INSERT INTO grades(student_id,evidence_id,grade) VALUES(?1,?2,?3)",
                            params![sid, eid, grade],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    for v in arr(root, "agenda") {
        tx.execute("INSERT INTO agenda(id,date,time,type,priority,text,done,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![sval(v,"id"),sval(v,"date"),sval(v,"time"),sval(v,"type"),sval(v,"priority"),sval(v,"text"),if v.get("done").and_then(Value::as_bool).unwrap_or(false){1}else{0},json(v)]).map_err(|e|e.to_string())?;
    }
    for v in arr(root, "meetings") {
        let mid = sval(v, "id");
        tx.execute("INSERT INTO meetings(id,date,student_id,type,start_time,end_time,title,development,agreements,followup,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![mid,sval(v,"date"),sval(v,"studentId"),sval(v,"type"),sval(v,"start"),sval(v,"end"),sval(v,"title"),sval(v,"development"),sval(v,"agreements"),sval(v,"followup"),json(v)]).map_err(|e|e.to_string())?;
        if let Some(a) = v.get("participants").and_then(Value::as_array) {
            for p in a {
                tx.execute("INSERT OR REPLACE INTO meeting_participants(meeting_id,name,status) VALUES(?1,?2,?3)",params![mid,sval(p,"name"),sval(p,"status")]).map_err(|e|e.to_string())?;
            }
        }
    }
    for p in arr(root, "meetingPeople") {
        if let Some(name) = p.as_str() {
            tx.execute(
                "INSERT OR IGNORE INTO meeting_people(name) VALUES(?1)",
                params![name],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    for v in arr(root, "customDays") {
        tx.execute(
            "INSERT INTO custom_days(id,start,end,type,label,raw_json) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                sval(v, "id"),
                sval(v, "start"),
                sval(v, "end"),
                sval(v, "type"),
                sval(v, "label"),
                json(v)
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    for v in arr(root, "issues") {
        tx.execute("INSERT INTO issues(id,created_at,updated_at,version,type,section,priority,title,description,expected,status,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![sval(v,"id"),sval(v,"createdAt"),sval(v,"updatedAt"),sval(v,"version"),sval(v,"type"),sval(v,"section"),sval(v,"priority"),sval(v,"title"),sval(v,"description"),sval(v,"expected"),sval(v,"status"),json(v)]).map_err(|e|e.to_string())?;
    }
    let settings = json(
        root.get("settings")
            .unwrap_or(&Value::Object(Default::default())),
    );
    tx.execute(
        "INSERT INTO settings(id,settings_json) VALUES(1,?1)",
        params![settings],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_state(app: AppHandle) -> Result<Option<String>, String> {
    let conn = open_db(&app)?;
    let mut stmt = conn
        .prepare("SELECT state_json FROM app_state WHERE id=1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(row.get(0).map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn save_state(app: AppHandle, state_json: String) -> Result<(), String> {
    let root: Value =
        serde_json::from_str(&state_json).map_err(|e| format!("JSON inválido: {e}"))?;
    let mut conn = open_db(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    sync_normalized(&tx, &root)?;
    let now = Utc::now().to_rfc3339();
    tx.execute("INSERT INTO app_state(id,state_json,updated_at) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET state_json=excluded.state_json,updated_at=excluded.updated_at",params![state_json,now]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn create_backup(app: AppHandle) -> Result<i64, String> {
    let mut conn = open_db(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let state: String = tx
        .query_row("SELECT state_json FROM app_state WHERE id=1", [], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO state_backups(state_json,created_at) VALUES(?1,?2)",
        params![state, Utc::now().to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    tx.commit().map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
fn database_status(app: AppHandle) -> Result<String, String> {
    let conn = open_db(&app)?;
    let path = db_path(&app)?;
    let ver: String = conn
        .query_row(
            "SELECT value FROM schema_meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "?".into());
    let students: i64 = conn
        .query_row("SELECT COUNT(*) FROM students", [], |r| r.get(0))
        .unwrap_or(0);
    let issues: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM issues WHERE status!='Resuelto'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Ok(format!(
        "SQLite · esquema {ver} · {students} alumnos · {issues} incidencias pendientes · {}",
        path.display()
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            load_state,
            save_state,
            create_backup,
            database_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running MiCuadernoDigital");
}
