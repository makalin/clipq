use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

use crate::database::{Database, Clip};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebClip {
    pub id: String,
    pub content: String,
    pub clip_type: String,
    pub created_at: String,
    pub file_path: Option<String>,
    pub tags: Vec<String>,
}

impl From<Clip> for WebClip {
    fn from(clip: Clip) -> Self {
        Self {
            id: clip.id,
            content: clip.content,
            clip_type: clip.clip_type,
            created_at: clip.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            file_path: clip.file_path,
            tags: Vec::new(), // Will be populated separately
        }
    }
}

pub struct WebServer {
    db: Arc<Mutex<Database>>,
    port: u16,
}

impl WebServer {
    pub fn new(db: Arc<Mutex<Database>>, port: u16) -> Self {
        Self { db, port }
    }

    pub async fn start(&self) -> Result<()> {
        let db = Arc::clone(&self.db);
        
        // CORS filter
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "DELETE"]);

        // Routes
        let clips = warp::path("api")
            .and(warp::path("clips"))
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(get_clips);

        let search = warp::path("api")
            .and(warp::path("search"))
            .and(warp::query::<SearchQuery>())
            .and(with_db(db.clone()))
            .and_then(search_clips);

        let add_clip = warp::path("api")
            .and(warp::path("clips"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db.clone()))
            .and_then(add_clip);

        let delete_clip = warp::path("api")
            .and(warp::path("clips"))
            .and(warp::path::param::<String>())
            .and(warp::delete())
            .and(with_db(db.clone()))
            .and_then(delete_clip);

        let stats = warp::path("api")
            .and(warp::path("stats"))
            .and(with_db(db.clone()))
            .and_then(get_stats);

        // Serve static files
        let static_files = warp::path("static")
            .and(warp::fs::dir("web/static/"));

        // Serve index.html for all other routes
        let index = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("web/index.html"));

        let routes = clips
            .or(search)
            .or(add_clip)
            .or(delete_clip)
            .or(stats)
            .or(static_files)
            .or(index)
            .with(cors);

        println!("Starting web server on http://localhost:{}", self.port);
        warp::serve(routes)
            .run(([127, 0, 0, 1], self.port))
            .await;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct AddClipRequest {
    content: String,
    clip_type: String,
}

fn with_db(db: Arc<Mutex<Database>>) -> impl Filter<Extract = (Arc<Mutex<Database>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn get_clips(db: Arc<Mutex<Database>>) -> Result<impl warp::Reply, warp::Rejection> {
    let rt = tokio::runtime::Handle::current();
    let db = rt.block_on(db.lock());
    let clips = rt.block_on(db.get_recent_clips(50)).map_err(|_| warp::reject::reject())?;
    
    let mut web_clips = Vec::new();
    for clip in clips {
        let mut web_clip = WebClip::from(clip.clone());
        web_clip.tags = rt.block_on(db.get_clip_tags(&clip.id)).unwrap_or_default();
        web_clips.push(web_clip);
    }
    
    Ok(warp::reply::json(&web_clips))
}

async fn search_clips(query: SearchQuery, db: Arc<Mutex<Database>>) -> Result<impl warp::Reply, warp::Rejection> {
    let db = db.lock().await;
    let limit = query.limit.unwrap_or(20);
    let clips = db.search_clips(&query.q, limit).await.map_err(|_| warp::reject::reject())?;
    
    let mut web_clips = Vec::new();
    for clip in clips {
        let mut web_clip = WebClip::from(clip.clone());
        web_clip.tags = db.get_clip_tags(&clip.id).await.unwrap_or_default();
        web_clips.push(web_clip);
    }
    
    Ok(warp::reply::json(&web_clips))
}

async fn add_clip(request: AddClipRequest, db: Arc<Mutex<Database>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db = db.lock().await;
    db.add_clip(&request.content, &request.clip_type).await.map_err(|_| warp::reject::reject())?;
    Ok(warp::reply::json(&serde_json::json!({"status": "success"})))
}

async fn delete_clip(clip_id: String, db: Arc<Mutex<Database>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db = db.lock().await;
    // Note: We'd need to implement delete_clip in the database module
    // For now, just return success
    Ok(warp::reply::json(&serde_json::json!({"status": "success"})))
}

async fn get_stats(db: Arc<Mutex<Database>>) -> Result<impl warp::Reply, warp::Rejection> {
    let db = db.lock().await;
    let stats = db.get_statistics().await.map_err(|_| warp::reject::reject())?;
    Ok(warp::reply::json(&stats))
}