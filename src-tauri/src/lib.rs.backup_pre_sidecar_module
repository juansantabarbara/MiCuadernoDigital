use chrono::{Duration as ChronoDuration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Europe::Madrid;
use rusqlite::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json as json_value, Value};
use std::{fs, io::{Read, Write}, net::TcpListener, path::PathBuf, process::Command, time::{Duration, Instant}};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use url::Url;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

const SCHEMA_VERSION: i64 = 5;
const GOOGLE_SCOPE: &str = "https://www.googleapis.com/auth/calendar.app.created";
const GOOGLE_CLIENT_ID: &str = env!("GOOGLE_CLIENT_ID");
const GOOGLE_CLIENT_SECRET: &str = env!("GOOGLE_CLIENT_SECRET");
const KEYCHAIN_SERVICE: &str = "MiCuadernoDigital Google Calendar";
const KEYCHAIN_SECRET_SERVICE: &str = "MiCuadernoDigital Google OAuth Client";

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("micuadernodigital.sqlite3"))
}

fn open_db(app: &AppHandle) -> Result<Connection, String> {
    let path = db_path(app)?;
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL").map_err(|e| e.to_string())?;
    conn.pragma_update(None, "foreign_keys", "ON").map_err(|e| e.to_string())?;
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
      CREATE TABLE IF NOT EXISTS actuaciones(id TEXT PRIMARY KEY,date TEXT,time TEXT,scope TEXT,requested_by TEXT,status TEXT,title TEXT,request_html TEXT,work_html TEXT,result_html TEXT,followup_html TEXT,updated_at TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_actuaciones_date ON actuaciones(date,time);
      CREATE TABLE IF NOT EXISTS aportaciones(id TEXT PRIMARY KEY,title TEXT NOT NULL,type TEXT,date TEXT,place TEXT,expected REAL,notes TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_aportaciones_date ON aportaciones(date);
      CREATE TABLE IF NOT EXISTS aportacion_pagos(aportacion_id TEXT NOT NULL,student_id TEXT NOT NULL,status TEXT,amount REAL,date TEXT,note TEXT,PRIMARY KEY(aportacion_id,student_id));
      CREATE TABLE IF NOT EXISTS custom_days(id TEXT PRIMARY KEY,start TEXT NOT NULL,end TEXT NOT NULL,type TEXT,label TEXT,raw_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS issues(id TEXT PRIMARY KEY,created_at TEXT,updated_at TEXT,version TEXT,type TEXT,section TEXT,priority TEXT,title TEXT,description TEXT,expected TEXT,status TEXT,raw_json TEXT NOT NULL);
      CREATE INDEX IF NOT EXISTS idx_issues_status_created ON issues(status,created_at);
      CREATE TABLE IF NOT EXISTS settings(id INTEGER PRIMARY KEY CHECK(id=1),settings_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS google_calendar_config(
        id INTEGER PRIMARY KEY CHECK(id=1),
        client_id TEXT NOT NULL,
        calendar_id TEXT NOT NULL,
        calendar_name TEXT NOT NULL DEFAULT 'MiCuadernoDigital',
        last_sync_at TEXT
      );
    "#).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO schema_meta(key,value) VALUES('schema_version',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![SCHEMA_VERSION.to_string()]).map_err(|e| e.to_string())?;
    Ok(conn)
}

fn sval(v: &Value, key: &str) -> String { v.get(key).and_then(Value::as_str).unwrap_or("").to_string() }
fn json(v: &Value) -> String { serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()) }
fn arr<'a>(root: &'a Value,key:&str)->Vec<&'a Value>{root.get(key).and_then(Value::as_array).map(|a|a.iter().collect()).unwrap_or_default()}

fn sync_normalized(tx:&Transaction<'_>, root:&Value)->Result<(),String>{
    for table in ["students","observations","attendance_entries","units","sessions","criteria","products","product_criteria","evidence_columns","grades","agenda","meetings","meeting_participants","meeting_people","actuaciones","aportacion_pagos","aportaciones","custom_days","issues","settings"] {
        tx.execute(&format!("DELETE FROM {table}"),[]).map_err(|e|e.to_string())?;
    }
    for v in arr(root,"students"){
        tx.execute("INSERT INTO students(id,name,status,attendance,feedbacks,coverage,strengths_json,difficulties_json,next_step,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![sval(v,"id"),sval(v,"name"),sval(v,"status"),v.get("attendance").and_then(Value::as_f64),v.get("feedbacks").and_then(Value::as_i64),v.get("coverage").and_then(Value::as_f64),json(v.get("strengths").unwrap_or(&Value::Null)),json(v.get("difficulties").unwrap_or(&Value::Null)),sval(v,"next"),json(v)]).map_err(|e|e.to_string())?;
    }
    for v in arr(root,"observations"){
        tx.execute("INSERT INTO observations(id,date,student_id,category,text,raw_json) VALUES(?1,?2,?3,?4,?5,?6)",params![sval(v,"id"),sval(v,"date"),sval(v,"studentId"),sval(v,"category"),sval(v,"text"),json(v)]).map_err(|e|e.to_string())?;
    }
    if let Some(map)=root.get("attendance").and_then(Value::as_object){
      for (date,rec) in map { if let Some(students)=rec.as_object(){ for (sid,status) in students { if let Some(st)=status.as_str(){tx.execute("INSERT INTO attendance_entries(date,student_id,status) VALUES(?1,?2,?3)",params![date,sid,st]).map_err(|e|e.to_string())?;}}}}
    }
    for u in arr(root,"units"){
      let uid=sval(u,"id");
      tx.execute("INSERT INTO units(id,title,areas,challenge,product,raw_json) VALUES(?1,?2,?3,?4,?5,?6)",params![uid,sval(u,"title"),sval(u,"areas"),sval(u,"challenge"),sval(u,"product"),json(u)]).map_err(|e|e.to_string())?;
      if let Some(a)=u.get("sessions").and_then(Value::as_array){for v in a{tx.execute("INSERT INTO sessions(id,unit_id,title,subject,date,start_time,end_time,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![sval(v,"id"),uid,sval(v,"title"),sval(v,"subject"),sval(v,"date"),sval(v,"startTime"),sval(v,"endTime"),json(v)]).map_err(|e|e.to_string())?;}}
      if let Some(a)=u.get("curricularCriteria").and_then(Value::as_array){for v in a{tx.execute("INSERT INTO criteria(id,unit_id,code,text,specific_competence,ce_text,descriptors,source,grade,area,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![sval(v,"id"),uid,sval(v,"code"),sval(v,"text"),sval(v,"ce"),sval(v,"ceText"),sval(v,"descriptors"),sval(v,"source"),sval(v,"grade"),sval(v,"area"),json(v)]).map_err(|e|e.to_string())?;}}
      if let Some(a)=u.get("products").and_then(Value::as_array){for v in a{let pid=sval(v,"id");tx.execute("INSERT INTO products(id,unit_id,name,area,raw_json) VALUES(?1,?2,?3,?4,?5)",params![pid,uid,sval(v,"name"),sval(v,"area"),json(v)]).map_err(|e|e.to_string())?;if let Some(ids)=v.get("criterionIds").and_then(Value::as_array){for cid in ids.iter().filter_map(Value::as_str){tx.execute("INSERT OR IGNORE INTO product_criteria(product_id,criterion_id) VALUES(?1,?2)",params![pid,cid]).map_err(|e|e.to_string())?;}}}}
    }
    for v in arr(root,"evidenceColumns"){
      tx.execute("INSERT INTO evidence_columns(id,label,criterion,criterion_text,specific_competence,descriptors,competency,unit_id,product_id,source,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![sval(v,"id"),sval(v,"label"),sval(v,"criterion"),sval(v,"criterionText"),sval(v,"specificCompetence"),sval(v,"descriptors"),sval(v,"competency"),sval(v,"unitId"),sval(v,"productId"),sval(v,"source"),json(v)]).map_err(|e|e.to_string())?;
    }
    if let Some(map)=root.get("evidence").and_then(Value::as_object){for (sid,rec) in map{if let Some(cols)=rec.as_object(){for (eid,g) in cols{if let Some(grade)=g.as_str(){tx.execute("INSERT INTO grades(student_id,evidence_id,grade) VALUES(?1,?2,?3)",params![sid,eid,grade]).map_err(|e|e.to_string())?;}}}}}
    for v in arr(root,"agenda"){tx.execute("INSERT INTO agenda(id,date,time,type,priority,text,done,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![sval(v,"id"),sval(v,"date"),sval(v,"time"),sval(v,"type"),sval(v,"priority"),sval(v,"text"),if v.get("done").and_then(Value::as_bool).unwrap_or(false){1}else{0},json(v)]).map_err(|e|e.to_string())?;}
    for v in arr(root,"meetings"){let mid=sval(v,"id");tx.execute("INSERT INTO meetings(id,date,student_id,type,start_time,end_time,title,development,agreements,followup,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![mid,sval(v,"date"),sval(v,"studentId"),sval(v,"type"),sval(v,"start"),sval(v,"end"),sval(v,"title"),sval(v,"development"),sval(v,"agreements"),sval(v,"followup"),json(v)]).map_err(|e|e.to_string())?;if let Some(a)=v.get("participants").and_then(Value::as_array){for p in a{tx.execute("INSERT OR REPLACE INTO meeting_participants(meeting_id,name,status) VALUES(?1,?2,?3)",params![mid,sval(p,"name"),if p.get("present").and_then(Value::as_bool).unwrap_or(true){"Presente"}else{"Ausente"}]).map_err(|e|e.to_string())?;}}}
    for p in arr(root,"meetingPeople"){if let Some(name)=p.as_str(){tx.execute("INSERT OR IGNORE INTO meeting_people(name) VALUES(?1)",params![name]).map_err(|e|e.to_string())?;}}
    for v in arr(root,"actuaciones"){tx.execute("INSERT INTO actuaciones(id,date,time,scope,requested_by,status,title,request_html,work_html,result_html,followup_html,updated_at,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",params![sval(v,"id"),sval(v,"date"),sval(v,"time"),sval(v,"scope"),sval(v,"requestedBy"),sval(v,"status"),sval(v,"title"),sval(v,"request"),sval(v,"work"),sval(v,"result"),sval(v,"followup"),sval(v,"updatedAt"),json(v)]).map_err(|e|e.to_string())?;}
    for v in arr(root,"aportaciones"){let aid=sval(v,"id");tx.execute("INSERT INTO aportaciones(id,title,type,date,place,expected,notes,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![aid,sval(v,"title"),sval(v,"type"),sval(v,"date"),sval(v,"place"),v.get("expected").and_then(Value::as_f64),sval(v,"notes"),json(v)]).map_err(|e|e.to_string())?;if let Some(payments)=v.get("payments").and_then(Value::as_object){for (sid,p) in payments{tx.execute("INSERT INTO aportacion_pagos(aportacion_id,student_id,status,amount,date,note) VALUES(?1,?2,?3,?4,?5,?6)",params![aid,sid,sval(p,"status"),p.get("amount").and_then(Value::as_f64),sval(p,"date"),sval(p,"note")]).map_err(|e|e.to_string())?;}}}
    for v in arr(root,"customDays"){tx.execute("INSERT INTO custom_days(id,start,end,type,label,raw_json) VALUES(?1,?2,?3,?4,?5,?6)",params![sval(v,"id"),sval(v,"start"),sval(v,"end"),sval(v,"type"),sval(v,"label"),json(v)]).map_err(|e|e.to_string())?;}
    for v in arr(root,"issues"){tx.execute("INSERT INTO issues(id,created_at,updated_at,version,type,section,priority,title,description,expected,status,raw_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![sval(v,"id"),sval(v,"createdAt"),sval(v,"updatedAt"),sval(v,"version"),sval(v,"type"),sval(v,"section"),sval(v,"priority"),sval(v,"title"),sval(v,"description"),sval(v,"expected"),sval(v,"status"),json(v)]).map_err(|e|e.to_string())?;}
    let settings=json(root.get("settings").unwrap_or(&Value::Object(Default::default())));
    tx.execute("INSERT INTO settings(id,settings_json) VALUES(1,?1)",params![settings]).map_err(|e|e.to_string())?;
    Ok(())
}


#[derive(Serialize)]
struct GoogleCalendarStatus {
    connected: bool,
    client_id: String,
    calendar_id: String,
    calendar_name: String,
    last_sync_at: String,
    detail: String,
}

#[derive(Serialize)]
struct GoogleSyncReport {
    created: usize,
    updated: usize,
    deleted: usize,
    total: usize,
    last_sync_at: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

fn token_entry(client_id: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYCHAIN_SERVICE, client_id)
        .map_err(|e| format!("No se pudo acceder al almacén seguro del sistema: {e}"))
}

fn keychain_store(client_id: &str, refresh_token: &str) -> Result<(), String> {
    token_entry(client_id)?
        .set_password(refresh_token)
        .map_err(|e| format!("No se pudo guardar la autorización de Google: {e}"))
}

fn keychain_get(client_id: &str) -> Result<String, String> {
    token_entry(client_id)?
        .get_password()
        .map_err(|_| "No hay una autorización de Google guardada.".to_string())
}

fn keychain_delete(client_id: &str) {
    if let Ok(entry) = token_entry(client_id) {
        let _ = entry.delete_credential();
    }
}

fn google_config(app: &AppHandle) -> Result<Option<(String,String,String,String)>, String> {
    let conn = open_db(app)?;
    let mut stmt = conn.prepare("SELECT client_id,calendar_id,calendar_name,COALESCE(last_sync_at,'') FROM google_calendar_config WHERE id=1").map_err(|e|e.to_string())?;
    let mut rows=stmt.query([]).map_err(|e|e.to_string())?;
    if let Some(r)=rows.next().map_err(|e|e.to_string())? {
        Ok(Some((r.get(0).map_err(|e|e.to_string())?,r.get(1).map_err(|e|e.to_string())?,r.get(2).map_err(|e|e.to_string())?,r.get(3).map_err(|e|e.to_string())?)))
    } else { Ok(None) }
}

fn random_token(len: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(len).map(char::from).collect()
}

fn refresh_access_token(client_id:&str, client_secret:&str, refresh_token:&str) -> Result<String,String> {
    let client=reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build().map_err(|e|e.to_string())?;

    let resp=client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id",client_id),
            ("client_secret",client_secret),
            ("refresh_token",refresh_token),
            ("grant_type","refresh_token")
        ])
        .send().map_err(|e|format!("No se pudo renovar la sesión de Google: {e}"))?;

    let status=resp.status();
    let text=resp.text().map_err(|e|e.to_string())?;

    if !status.is_success(){
        return Err(format!("Google rechazó la renovación ({status}): {text}"));
    }

    let v:Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
    v.get("access_token")
        .and_then(Value::as_str)
        .map(|x|x.to_string())
        .ok_or_else(||"Google no devolvió un access_token.".into())
}

fn create_google_calendar(access_token:&str)->Result<String,String>{
    let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(30)).build().map_err(|e|e.to_string())?;
    let resp=client.post("https://www.googleapis.com/calendar/v3/calendars")
      .bearer_auth(access_token)
      .json(&json_value!({"summary":"MiCuadernoDigital","description":"Agenda sincronizada desde MiCuadernoDigital","timeZone":"Europe/Madrid"}))
      .send().map_err(|e|format!("No se pudo crear el calendario: {e}"))?;
    let status=resp.status();let text=resp.text().map_err(|e|e.to_string())?;
    if !status.is_success(){return Err(format!("Google Calendar respondió {status}: {text}"));}
    let v:Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
    v.get("id").and_then(Value::as_str).map(|x|x.to_string()).ok_or_else(||"Google no devolvió el ID del calendario.".into())
}

fn oauth_connect_blocking(app:AppHandle)->Result<GoogleCalendarStatus,String>{
    let client_id = GOOGLE_CLIENT_ID.to_string();
    let listener=TcpListener::bind("127.0.0.1:0").map_err(|e|format!("No se pudo abrir el puerto local de autorización: {e}"))?;
    listener.set_nonblocking(true).map_err(|e|e.to_string())?;
    let port=listener.local_addr().map_err(|e|e.to_string())?.port();
    let redirect_uri=format!("http://127.0.0.1:{port}");
    let verifier=random_token(64);
    let challenge=URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state=random_token(32);
    let mut auth=Url::parse("https://accounts.google.com/o/oauth2/v2/auth").map_err(|e|e.to_string())?;
    auth.query_pairs_mut()
      .append_pair("client_id",client_id.trim())
      .append_pair("redirect_uri",&redirect_uri)
      .append_pair("response_type","code")
      .append_pair("scope",GOOGLE_SCOPE)
      .append_pair("access_type","offline")
      .append_pair("prompt","consent")
      .append_pair("code_challenge",&challenge)
      .append_pair("code_challenge_method","S256")
      .append_pair("state",&state);
    open::that(auth.as_str())
        .map_err(|e|format!("No se pudo abrir el navegador para conectar Google: {e}"))?;
    let started=Instant::now();
    let (mut stream,_)=loop {
      match listener.accept(){
        Ok(v)=>break v,
        Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>{if started.elapsed()>Duration::from_secs(180){return Err("La autorización de Google agotó el tiempo de espera.".into());}std::thread::sleep(Duration::from_millis(100));},
        Err(e)=>return Err(format!("No se recibió la respuesta de Google: {e}")),
      }
    };
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    let mut buf=[0u8;8192];let n=stream.read(&mut buf).map_err(|e|format!("No se pudo leer la respuesta de Google: {e}"))?;
    let req=String::from_utf8_lossy(&buf[..n]);
    let target=req.lines().next().and_then(|l|l.split_whitespace().nth(1)).ok_or_else(||"Respuesta OAuth inválida.".to_string())?;
    let callback=Url::parse(&format!("http://127.0.0.1{target}")).map_err(|e|e.to_string())?;
    let params:std::collections::HashMap<String,String>=callback.query_pairs().map(|(k,v)|(k.into_owned(),v.into_owned())).collect();
    let response_html="<html><body style='font-family:-apple-system;padding:40px'><h2>MiCuadernoDigital conectado</h2><p>Puedes cerrar esta pestaña y volver a la aplicación.</p></body></html>";
    let response=format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",response_html.as_bytes().len(),response_html);
    let _=stream.write_all(response.as_bytes());
    if params.get("state")!=Some(&state){return Err("La respuesta de Google no superó la comprobación de seguridad (state).".into());}
    if let Some(err)=params.get("error"){return Err(format!("Google canceló la autorización: {err}"));}
    let code=params.get("code").ok_or_else(||"Google no devolvió el código de autorización.".to_string())?;
    let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(30)).build().map_err(|e|e.to_string())?;
    let resp=client.post("https://oauth2.googleapis.com/token")
      .form(&[("client_id",client_id.trim()),("client_secret",GOOGLE_CLIENT_SECRET),("code",code.as_str()),("code_verifier",verifier.as_str()),("grant_type","authorization_code"),("redirect_uri",redirect_uri.as_str())])
      .send().map_err(|e|format!("No se pudo completar la autorización: {e}"))?;
    let status=resp.status();let text=resp.text().map_err(|e|e.to_string())?;
    if !status.is_success(){return Err(format!("Google OAuth respondió {status}: {text}"));}
    let tok:TokenResponse=serde_json::from_str(&text).map_err(|e|format!("Respuesta OAuth inválida: {e}"))?;
    let refresh=tok.refresh_token.ok_or_else(||"Google no devolvió refresh_token. Revoca el acceso anterior de MiCuadernoDigital en tu cuenta de Google y vuelve a conectar.".to_string())?;
    let previous=google_config(&app)?;
    if let Some((old_client,_,_,_))=&previous { if old_client!=client_id.trim(){keychain_delete(old_client);} }
    keychain_store(client_id.trim(),&refresh)?;
    let calendar_id=match previous.filter(|(old_client,cal,_,_)|old_client==client_id.trim()&&!cal.is_empty()).map(|(_,cal,_,_)|cal){Some(cal)=>cal,None=>create_google_calendar(&tok.access_token)?};
    let conn=open_db(&app)?;
    conn.execute("INSERT INTO google_calendar_config(id,client_id,calendar_id,calendar_name,last_sync_at) VALUES(1,?1,?2,'MiCuadernoDigital','') ON CONFLICT(id) DO UPDATE SET client_id=excluded.client_id,calendar_id=excluded.calendar_id,calendar_name=excluded.calendar_name,last_sync_at=''",params![client_id.trim(),calendar_id]).map_err(|e|e.to_string())?;
    Ok(GoogleCalendarStatus{connected:true,client_id:client_id.trim().into(),calendar_id,calendar_name:"MiCuadernoDigital".into(),last_sync_at:"".into(),detail:"Conectado. Se ha creado un calendario independiente llamado MiCuadernoDigital.".into()})
}

#[tauri::command]
async fn google_connect(app:AppHandle)->Result<GoogleCalendarStatus,String>{
    let result = tauri::async_runtime::spawn_blocking(move||oauth_connect_blocking(app))
        .await
        .map_err(|e|e.to_string())?;

    if let Err(ref e) = result {
        eprintln!("GOOGLE_CONNECT_ERROR: {e}");
    }

    result
}

#[tauri::command]
fn google_status(app:AppHandle)->Result<GoogleCalendarStatus,String>{
    if let Some((client_id,calendar_id,calendar_name,last_sync_at))=google_config(&app)?{
        let connected=keychain_get(&client_id).is_ok();
        Ok(GoogleCalendarStatus{connected,client_id,calendar_id,calendar_name,last_sync_at,detail:if connected{"Google Calendar conectado.".into()}else{"Existe configuración, pero falta la autorización segura de Google.".into()}})
    } else {Ok(GoogleCalendarStatus{connected:false,client_id:"".into(),calendar_id:"".into(),calendar_name:"MiCuadernoDigital".into(),last_sync_at:"".into(),detail:"No conectado.".into()})}
}

#[tauri::command]
fn google_disconnect(app:AppHandle)->Result<(),String>{
    if let Some((client_id,_,_,_))=google_config(&app)?{
        keychain_delete(&client_id);
    }
    Ok(())
}

fn agenda_event_body(item:&Value,color_map:&Value,default_duration:i64)->Result<Value,String>{
    let id=sval(item,"id");let date=sval(item,"date");let time=sval(item,"time");let kind=sval(item,"type");let priority=sval(item,"priority");let text=sval(item,"text");let done=item.get("done").and_then(Value::as_bool).unwrap_or(false);
    if id.is_empty()||date.is_empty()||text.is_empty(){return Err("Hay un elemento de agenda incompleto.".into());}
    let color=color_map.get(&kind).and_then(Value::as_str).unwrap_or("8");
    let summary=format!("{}{} · {}",if done{"✓ "}else{""},kind,text);
    let description=format!("Creado desde MiCuadernoDigital\nTipo: {kind}\nPrioridad: {priority}\nID local: {id}");
    let (start,end)=if time.is_empty(){
      let d=NaiveDate::parse_from_str(&date,"%Y-%m-%d").map_err(|_|"Fecha de agenda inválida.".to_string())?;
      (json_value!({"date":d.format("%Y-%m-%d").to_string()}),json_value!({"date":(d+ChronoDuration::days(1)).format("%Y-%m-%d").to_string()}))
    }else{
      let d=NaiveDate::parse_from_str(&date,"%Y-%m-%d").map_err(|_|"Fecha de agenda inválida.".to_string())?;
      let t=NaiveTime::parse_from_str(&time,"%H:%M").map_err(|_|"Hora de agenda inválida.".to_string())?;
      let naive=NaiveDateTime::new(d,t);let start_dt=Madrid.from_local_datetime(&naive).earliest().ok_or_else(||"No se pudo interpretar la hora en Europe/Madrid.".to_string())?;
      let end_dt=start_dt+ChronoDuration::minutes(default_duration.max(5));
      (json_value!({"dateTime":start_dt.to_rfc3339(),"timeZone":"Europe/Madrid"}),json_value!({"dateTime":end_dt.to_rfc3339(),"timeZone":"Europe/Madrid"}))
    };
    let hash=Sha256::digest(id.as_bytes());let hex:String=hash.iter().take(16).map(|b|format!("{:02x}",b)).collect();let event_id=format!("mcd{hex}");
    Ok(json_value!({"id":event_id,"summary":summary,"description":description,"colorId":color,"start":start,"end":end,"extendedProperties":{"private":{"source":"MiCuadernoDigital","agendaId":id,"agendaType":kind}}}))
}

fn google_sync_blocking(app:AppHandle,agenda_json:String,color_map_json:String,default_duration_minutes:i64)->Result<GoogleSyncReport,String>{
    let (client_id,calendar_id,_,_)=google_config(&app)?.ok_or_else(||"Google Calendar no está conectado.".to_string())?;
    let refresh=keychain_get(&client_id)?;
    let access=refresh_access_token(&client_id,GOOGLE_CLIENT_SECRET,&refresh)?;
    let agenda:Value=serde_json::from_str(&agenda_json).map_err(|e|format!("Agenda inválida: {e}"))?;
    let items=agenda.as_array().ok_or_else(||"La agenda no tiene el formato esperado.".to_string())?;
    let colors:Value=serde_json::from_str(&color_map_json).unwrap_or_else(|_|json_value!({}));
    let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(30)).build().map_err(|e|e.to_string())?;
    let cal=urlencoding::encode(&calendar_id);
    let list_url=format!("https://www.googleapis.com/calendar/v3/calendars/{cal}/events");
    let resp=client.get(&list_url).bearer_auth(&access).query(&[("privateExtendedProperty","source=MiCuadernoDigital"),("maxResults","2500"),("showDeleted","false")]).send().map_err(|e|format!("No se pudieron leer los eventos sincronizados: {e}"))?;
    let status=resp.status();let text=resp.text().map_err(|e|e.to_string())?;
    if !status.is_success(){return Err(format!("Google Calendar respondió {status}: {text}"));}
    let listed:Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
    let mut existing=std::collections::HashMap::<String,String>::new();
    if let Some(arr)=listed.get("items").and_then(Value::as_array){for ev in arr{if let (Some(eid),Some(aid))=(ev.get("id").and_then(Value::as_str),ev.get("extendedProperties").and_then(|x|x.get("private")).and_then(|x|x.get("agendaId")).and_then(Value::as_str)){existing.insert(aid.into(),eid.into());}}}
    let mut created=0usize;let mut updated=0usize;let mut local_ids=std::collections::HashSet::new();
    for item in items{
      let local_id=sval(item,"id");if local_id.is_empty(){continue}local_ids.insert(local_id.clone());
      let body=agenda_event_body(item,&colors,default_duration_minutes)?;
      let event_id=body.get("id").and_then(Value::as_str).unwrap();
      if let Some(existing_event_id)=existing.get(&local_id){
        let url=format!("{list_url}/{}",urlencoding::encode(existing_event_id));let r=client.put(&url).bearer_auth(&access).json(&body).send().map_err(|e|e.to_string())?;let st=r.status();let tx=r.text().unwrap_or_default();if !st.is_success(){return Err(format!("No se pudo actualizar un evento ({st}): {tx}"));}updated+=1;
      }else{
        let r=client.post(&list_url).bearer_auth(&access).json(&body).send().map_err(|e|e.to_string())?;let st=r.status();let tx=r.text().unwrap_or_default();
        if st.as_u16()==409{let url=format!("{list_url}/{event_id}");let rr=client.put(&url).bearer_auth(&access).json(&body).send().map_err(|e|e.to_string())?;let ss=rr.status();let tt=rr.text().unwrap_or_default();if !ss.is_success(){return Err(format!("No se pudo reconciliar un evento ({ss}): {tt}"));}updated+=1;}else if !st.is_success(){return Err(format!("No se pudo crear un evento ({st}): {tx}"));}else{created+=1;}
      }
    }
    let mut deleted=0usize;
    for (agenda_id,event_id) in existing{if !local_ids.contains(&agenda_id){let url=format!("{list_url}/{}",urlencoding::encode(&event_id));let r=client.delete(&url).bearer_auth(&access).send().map_err(|e|e.to_string())?;if r.status().is_success()||r.status().as_u16()==410||r.status().as_u16()==404{deleted+=1;}else{return Err(format!("No se pudo borrar un evento antiguo: {}",r.status()));}}}
    let now=Utc::now().to_rfc3339();let conn=open_db(&app)?;conn.execute("UPDATE google_calendar_config SET last_sync_at=?1 WHERE id=1",params![now]).map_err(|e|e.to_string())?;
    Ok(GoogleSyncReport{created,updated,deleted,total:items.len(),last_sync_at:now})
}

#[tauri::command]
async fn google_sync_agenda(app:AppHandle,agenda_json:String,color_map_json:String,default_duration_minutes:i64)->Result<GoogleSyncReport,String>{
    tauri::async_runtime::spawn_blocking(move||google_sync_blocking(app,agenda_json,color_map_json,default_duration_minutes)).await.map_err(|e|e.to_string())?
}

#[derive(Serialize)]
struct AppUpdateStatus {
    available: bool,
    version: String,
    notes: String,
    current_version: String,
}

#[tauri::command]
async fn check_for_update(app: AppHandle) -> Result<AppUpdateStatus, String> {
    let current_version = app.package_info().version.to_string();
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => Ok(AppUpdateStatus {
            available: true,
            version: update.version.to_string(),
            notes: update.body.clone().unwrap_or_default(),
            current_version,
        }),
        None => Ok(AppUpdateStatus {
            available: false,
            version: current_version.clone(),
            notes: String::new(),
            current_version,
        }),
    }
}

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        update.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}

#[tauri::command]
fn load_state(app: AppHandle) -> Result<Option<String>, String> {
    let conn=open_db(&app)?;
    let mut stmt=conn.prepare("SELECT state_json FROM app_state WHERE id=1").map_err(|e|e.to_string())?;
    let mut rows=stmt.query([]).map_err(|e|e.to_string())?;
    if let Some(row)=rows.next().map_err(|e|e.to_string())?{Ok(Some(row.get(0).map_err(|e|e.to_string())?))}else{Ok(None)}
}

#[tauri::command]
fn save_state(app: AppHandle,state_json:String)->Result<(),String>{
    let root:Value=serde_json::from_str(&state_json).map_err(|e|format!("JSON inválido: {e}"))?;
    let mut conn=open_db(&app)?;
    let tx=conn.transaction().map_err(|e|e.to_string())?;
    sync_normalized(&tx,&root)?;
    let now=Utc::now().to_rfc3339();
    tx.execute("INSERT INTO app_state(id,state_json,updated_at) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET state_json=excluded.state_json,updated_at=excluded.updated_at",params![state_json,now]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(())
}

#[tauri::command]
fn create_backup(app:AppHandle)->Result<i64,String>{
    let mut conn=open_db(&app)?;let tx=conn.transaction().map_err(|e|e.to_string())?;
    let state:String=tx.query_row("SELECT state_json FROM app_state WHERE id=1",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO state_backups(state_json,created_at) VALUES(?1,?2)",params![state,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    let id=tx.last_insert_rowid();tx.commit().map_err(|e|e.to_string())?;Ok(id)
}

#[tauri::command]
fn database_status(app:AppHandle)->Result<String,String>{
    let conn=open_db(&app)?;let path=db_path(&app)?;
    let ver:String=conn.query_row("SELECT value FROM schema_meta WHERE key='schema_version'",[],|r|r.get(0)).unwrap_or_else(|_|"?".into());
    let students:i64=conn.query_row("SELECT COUNT(*) FROM students",[],|r|r.get(0)).unwrap_or(0);
    let issues:i64=conn.query_row("SELECT COUNT(*) FROM issues WHERE status!='Resuelto'",[],|r|r.get(0)).unwrap_or(0);
    Ok(format!("SQLite · esquema {ver} · {students} alumnos · {issues} incidencias pendientes · {}",path.display()))
}


#[tauri::command]
fn generate_aportacion_csv(
    app: AppHandle,
    aportacion_json: String,
    students_json: String,
) -> Result<String, String> {
    let aportacion: serde_json::Value =
        serde_json::from_str(&aportacion_json).map_err(|e| e.to_string())?;

    let students: Vec<serde_json::Value> =
        serde_json::from_str(&students_json).map_err(|e| e.to_string())?;

    fn val(v: Option<&serde_json::Value>) -> String {
        match v {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::Bool(b)) => b.to_string(),
            _ => String::new(),
        }
    }

    fn csv_cell(s: &str) -> String {
        let cleaned = s.replace('\r', " ").replace('\n', " ");
        if cleaned.contains(';') || cleaned.contains('"') {
            format!("\"{}\"", cleaned.replace('"', "\"\""))
        } else {
            cleaned
        }
    }

    fn csv_number(v: f64) -> String {
        format!("{:.2}", v).replace('.', ",")
    }

    let title = val(aportacion.get("title"));
    let kind = val(aportacion.get("type"));
    let date = val(aportacion.get("date"));
    let place = val(aportacion.get("place"));
    let expected = aportacion
        .get("expected")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let payments = aportacion
        .get("payments")
        .and_then(|v| v.as_object());

    let mut rows = String::new();

    // BOM UTF-8 para que Excel reconozca correctamente tildes y ñ.
    rows.push('\u{feff}');
    rows.push_str(
        "Aportación;Tipo;Fecha aportación;Lugar;Alumno/a;Estado;Importe entregado;Importe previsto;Fecha pago;Observación\n"
    );

    let mut ordered_students = students;
    ordered_students.sort_by(|a, b| {
        val(a.get("name"))
            .to_lowercase()
            .cmp(&val(b.get("name")).to_lowercase())
    });

    for student in ordered_students {
        let student_id = val(student.get("id"));
        let student_name = val(student.get("name"));

        let payment = payments.and_then(|p| p.get(&student_id));

        let explicit_status = payment
            .and_then(|p| p.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let amount = payment
            .and_then(|p| p.get("amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let status = if explicit_status == "Pagado" {
            "Pagado"
        } else if amount > 0.0 {
            "Parcial"
        } else {
            "Pendiente"
        };

        let payment_date = payment
            .map(|p| val(p.get("date")))
            .unwrap_or_default();

        let note = payment
            .map(|p| val(p.get("note")))
            .unwrap_or_default();

        let fields = [
            csv_cell(&title),
            csv_cell(&kind),
            csv_cell(&date),
            csv_cell(&place),
            csv_cell(&student_name),
            csv_cell(status),
            csv_number(amount),
            csv_number(expected),
            csv_cell(&payment_date),
            csv_cell(&note),
        ];

        rows.push_str(&fields.join(";"));
        rows.push('\n');
    }

    let mut safe_name: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    while safe_name.contains("__") {
        safe_name = safe_name.replace("__", "_");
    }

    safe_name = safe_name.trim_matches('_').to_string();

    if safe_name.is_empty() {
        safe_name = "aportacion".to_string();
    }

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(format!("aportaciones_{}.csv", safe_name));

    std::fs::write(&path, rows.as_bytes()).map_err(|e| e.to_string())?;

    open::that(&path).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn generate_aportacion_pdf(
    app: AppHandle,
    aportacion_json: String,
    students_json: String,
) -> Result<String, String> {
    use printpdf::*;
    use std::io::BufWriter;

    let a: Value = serde_json::from_str(&aportacion_json)
        .map_err(|e| format!("Datos de la aportación inválidos: {e}"))?;

    let students: Value = serde_json::from_str(&students_json)
        .map_err(|e| format!("Datos del alumnado inválidos: {e}"))?;

    let students = students
        .as_array()
        .ok_or_else(|| "El alumnado no tiene el formato esperado.".to_string())?;

    fn text(v: &Value, key: &str) -> String {
        v.get(key).and_then(Value::as_str).unwrap_or("").trim().to_string()
    }

    fn format_date(value: &str) -> String {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|d| d.format("%d/%m/%Y").to_string())
            .unwrap_or_else(|_| value.to_string())
    }

    fn safe_pdf_text(value: &str) -> String {
        value
            .replace('€', "EUR")
            .replace('–', "-")
            .replace('—', "-")
            .replace('“', "\"")
            .replace('”', "\"")
            .replace('’', "'")
    }

    let title = text(&a, "title");
    let date = format_date(&text(&a, "date"));
    let place = text(&a, "place");
    let stage = text(&a, "stage");
    let cycle = text(&a, "cycle");
    let level = text(&a, "level");
    let group = text(&a, "group");
    let locality = text(&a, "locality");
    let document_date = format_date(&text(&a, "documentDate"));

    let payments = a
        .get("payments")
        .and_then(Value::as_object);

    let mut paid_students: Vec<(String, f64)> = Vec::new();

    for student in students {
        let sid = text(student, "id");
        let name = text(student, "name");

        if sid.is_empty() || name.is_empty() {
            continue;
        }

        let Some(payment) = payments.and_then(|p| p.get(&sid)) else {
            continue;
        };

        let status = payment
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("");

        if status != "Pagado" {
            continue;
        }

        let amount = payment
            .get("amount")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        paid_students.push((name, amount));
    }

    paid_students.sort_by(|a, b| {
        a.0.to_lowercase().cmp(&b.0.to_lowercase())
    });

    let total: f64 = paid_students.iter().map(|(_, amount)| *amount).sum();

    let (doc, page1, layer1) =
        PdfDocument::new("Aportaciones alumnado", Mm(210.0), Mm(297.0), "Documento");

    let regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("No se pudo preparar la fuente del PDF: {e}"))?;

    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("No se pudo preparar la fuente del PDF: {e}"))?;

    let mut page = page1;
    let mut layer = layer1;
    let mut y = 280.0_f32;

    fn write_line(
        doc: &PdfDocumentReference,
        page: PdfPageIndex,
        layer: PdfLayerIndex,
        font: &IndirectFontRef,
        value: &str,
        size: f32,
        x: f32,
        y: f32,
    ) {
        doc.get_page(page)
            .get_layer(layer)
            .use_text(value, size, Mm(x), Mm(y), font);
    }

    fn new_page(
        doc: &PdfDocumentReference,
    ) -> (PdfPageIndex, PdfLayerIndex) {
        doc.add_page(Mm(210.0), Mm(297.0), "Documento")
    }

    // Cabecera del centro, conservando el formato del documento original.
    write_line(&doc, page, layer, &bold, "C.P. SANTA BARBARA", 12.0, 18.0, y);
    y -= 5.5;
    write_line(&doc, page, layer, &regular, "C/ El Resbalon s/n - 33420 - LUGONES", 8.5, 18.0, y);
    y -= 4.5;
    write_line(&doc, page, layer, &regular, "E-MAIL: santabar@educastur.org", 8.5, 18.0, y);

    y -= 14.0;

    write_line(
        &doc, page, layer, &bold,
        "FICHA APORTACIONES ALUMNADO SALIDA",
        14.0, 37.0, y
    );
    y -= 7.0;
    write_line(
        &doc, page, layer, &bold,
        "COMPLEMENTARIA/EXTRAESCOLAR",
        14.0, 48.0, y
    );

    y -= 15.0;

    let school_year = {
        let today = Utc::now().with_timezone(&Madrid).date_naive();
        let year = today.format("%Y").to_string().parse::<i32>().unwrap_or(0);
        let month = today.format("%m").to_string().parse::<u32>().unwrap_or(1);

        if month >= 9 {
            format!("CURSO {}/{}", year, year + 1)
        } else {
            format!("CURSO {}/{}", year - 1, year)
        }
    };

    write_line(&doc, page, layer, &bold, &school_year, 10.0, 18.0, y);
    y -= 10.0;

    let row1 = format!(
        "ETAPA: {}     CICLO/INTERNIVEL: {}     NIVEL: {}     GRUPO: {}",
        safe_pdf_text(&stage),
        safe_pdf_text(&cycle),
        safe_pdf_text(&level),
        safe_pdf_text(&group)
    );
    write_line(&doc, page, layer, &regular, &row1, 9.0, 18.0, y);

    y -= 8.0;
    write_line(
        &doc, page, layer, &bold,
        &format!("DENOMINACION: {}", safe_pdf_text(&title)),
        9.5, 18.0, y
    );

    y -= 8.0;
    write_line(
        &doc, page, layer, &regular,
        &format!("FECHA: {}     LUGAR: {}", safe_pdf_text(&date), safe_pdf_text(&place)),
        9.0, 18.0, y
    );

    y -= 14.0;

    write_line(&doc, page, layer, &bold, "ALUMNO/A", 9.0, 20.0, y);
    write_line(&doc, page, layer, &bold, "APORTACION ECONOMICA", 9.0, 130.0, y);
    y -= 3.0;

    // Línea bajo la cabecera de la tabla.
    {
        let current_layer = doc.get_page(page).get_layer(layer);
        let line = Line {
            points: vec![
                (Point::new(Mm(18.0), Mm(y)), false),
                (Point::new(Mm(192.0), Mm(y)), false),
            ],
            is_closed: false,
        };
        current_layer.add_line(line);
    }

    y -= 7.0;

    if paid_students.is_empty() {
        write_line(
            &doc, page, layer, &regular,
            "No hay alumnado marcado como Pagado.",
            9.0, 20.0, y
        );
        y -= 8.0;
    } else {
        for (name, amount) in &paid_students {
            if y < 35.0 {
                let next = new_page(&doc);
                page = next.0;
                layer = next.1;
                y = 278.0;

                write_line(&doc, page, layer, &bold, "ALUMNO/A", 9.0, 20.0, y);
                write_line(&doc, page, layer, &bold, "APORTACION ECONOMICA", 9.0, 130.0, y);
                y -= 9.0;
            }

            let amount_text = format!("{:.2} EUR", amount).replace('.', ",");

            write_line(
                &doc, page, layer, &regular,
                &safe_pdf_text(name),
                9.0, 20.0, y
            );

            write_line(
                &doc, page, layer, &regular,
                &amount_text,
                9.0, 145.0, y
            );

            y -= 7.0;
        }
    }

    if y < 55.0 {
        let next = new_page(&doc);
        page = next.0;
        layer = next.1;
        y = 278.0;
    }

    y -= 4.0;

    {
        let current_layer = doc.get_page(page).get_layer(layer);
        let line = Line {
            points: vec![
                (Point::new(Mm(18.0), Mm(y)), false),
                (Point::new(Mm(192.0), Mm(y)), false),
            ],
            is_closed: false,
        };
        current_layer.add_line(line);
    }

    y -= 8.0;

    let total_text = format!("TOTAL: {:.2} EUR", total).replace('.', ",");
    write_line(&doc, page, layer, &bold, &total_text, 10.0, 130.0, y);

    y -= 18.0;

    let signature = if locality.is_empty() && document_date.is_empty() {
        String::new()
    } else if document_date.is_empty() {
        locality
    } else if locality.is_empty() {
        document_date
    } else {
        format!("{}, {}", locality, document_date)
    };

    if !signature.is_empty() {
        write_line(
            &doc, page, layer, &regular,
            &safe_pdf_text(&signature),
            9.0, 20.0, y
        );
        y -= 14.0;
    }

    write_line(&doc, page, layer, &regular, "Firmado:", 9.0, 20.0, y);

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo localizar la carpeta de la aplicacion: {e}"))?;

    fs::create_dir_all(&dir)
        .map_err(|e| format!("No se pudo preparar la carpeta del PDF: {e}"))?;

    let filename = if title.trim().is_empty() {
        "aportaciones.pdf".to_string()
    } else {
        let clean: String = title
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else if c.is_whitespace() {
                    '_'
                } else {
                    '_'
                }
            })
            .collect();

        format!("aportaciones_{}.pdf", clean)
    };

    let path = dir.join(filename);

    let file = fs::File::create(&path)
        .map_err(|e| format!("No se pudo crear el PDF: {e}"))?;

    doc.save(&mut BufWriter::new(file))
        .map_err(|e| format!("No se pudo guardar el PDF: {e}"))?;

    open::that(&path)
        .map_err(|e| format!("El PDF se creo, pero no se pudo abrir: {e}"))?;

    Ok(path.to_string_lossy().to_string())
}

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
    tauri::Builder::default()
      .plugin(tauri_plugin_updater::Builder::new().build())
      .invoke_handler(tauri::generate_handler![load_state,save_state,create_backup,database_status,google_connect,google_status,google_disconnect,google_sync_agenda,check_for_update,install_update,generate_aportacion_pdf,generate_aportacion_csv])
      .run(tauri::generate_context!())
      .expect("error while running MiCuadernoDigital");
}
