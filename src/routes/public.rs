use crate::error::{AppError, AppResult};
use crate::models::{CreateMessage, Experience, Message, Post, Project, Skill};
use crate::services::email::EmailService;
use crate::services::github::GitHubService;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use tera::{Context, Tera};
use tracing::warn;

const CACHE_PUBLIC_60S: (&str, &str) = ("Cache-Control", "public, max-age=60");

pub fn public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/", web::get().to(home))
            .route("/projects", web::get().to(projects))
            .route("/projects/{id}", web::get().to(project_detail))
            .route("/blog", web::get().to(blog))
            .route("/blog/{slug}", web::get().to(post_detail))
            .route("/about", web::get().to(about))
            .route("/contact", web::get().to(contact))
            .route("/contact", web::post().to(contact_submit)),
    );
}

pub async fn not_found(req: HttpRequest, tmpl: web::Data<Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("page_title", "Page not found - Kadyn Pearce");
    ctx.insert("path", req.path());

    let body = tmpl
        .render("pages/404.html", &ctx)
        .unwrap_or_else(|_| {
            r#"<!DOCTYPE html><html><body><h1>404</h1><p>Page not found.</p><a href="/">Home</a></body></html>"#.into()
        });

    HttpResponse::NotFound()
        .content_type("text/html")
        .body(body)
}

async fn home(tmpl: web::Data<Tera>) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();

    let (featured_projects, recent_posts, skills, github_stats) = tokio::join!(
        Project::featured(),
        Post::recent(3),
        Skill::all(),
        async { GitHubService::get_user_stats("kadynjaipearce").await.ok() },
    );

    let featured_projects = featured_projects.unwrap_or_default();
    let recent_posts = recent_posts.unwrap_or_default();
    let skills = skills.unwrap_or_default();

    ctx.insert("featured_projects", &featured_projects);
    ctx.insert("recent_posts", &recent_posts);
    ctx.insert("skills", &skills);
    ctx.insert("github_stats", &github_stats);
    ctx.insert("page_title", "Kadyn Pearce – Software Engineer");
    ctx.insert(
        "page_description",
        "Rust-first systems engineer building trading, infrastructure, and ML workloads for production.",
    );

    let body = tmpl.render("pages/home.html", &ctx).map_err(|e| {
        tracing::error!("Template render error: {:?}", e);
        AppError::TemplateError(format!("{:?}", e))
    })?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .insert_header(CACHE_PUBLIC_60S)
        .body(body))
}

#[derive(Deserialize)]
pub struct ProjectsQuery {
    category: Option<String>,
}

async fn projects(
    tmpl: web::Data<Tera>,
    query: web::Query<ProjectsQuery>,
) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();

    let projects = if let Some(ref cat) = query.category {
        Project::by_project_type(cat).await?
    } else {
        Project::all().await?
    };

    ctx.insert("projects", &projects);
    ctx.insert("current_category", &query.category);
    ctx.insert("page_title", "Projects - Kadyn Pearce");

    let body = tmpl
        .render("pages/projects.html", &ctx)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .insert_header(CACHE_PUBLIC_60S)
        .body(body))
}

async fn project_detail(tmpl: web::Data<Tera>, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let mut ctx = Context::new();

    let project = Project::by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let content_html = {
        use pulldown_cmark::{html, Parser};
        let parser = Parser::new(&project.description);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    };

    ctx.insert("project", &project);
    ctx.insert("content_html", &content_html);
    ctx.insert("page_title", &format!("{} - Kadyn Pearce", project.name));

    let body = tmpl
        .render("pages/project_detail.html", &ctx)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .insert_header(CACHE_PUBLIC_60S)
        .body(body))
}

#[derive(Deserialize)]
pub struct BlogQuery {
    tag: Option<String>,
}

async fn blog(tmpl: web::Data<Tera>, query: web::Query<BlogQuery>) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();

    let posts = if let Some(ref tag) = query.tag {
        Post::by_tag(tag).await?
    } else {
        Post::published().await?
    };

    ctx.insert("posts", &posts);
    ctx.insert("current_tag", &query.tag);
    ctx.insert("page_title", "Blog - Kadyn Pearce");

    let body = tmpl
        .render("pages/blog.html", &ctx)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .insert_header(CACHE_PUBLIC_60S)
        .body(body))
}

async fn post_detail(tmpl: web::Data<Tera>, path: web::Path<String>) -> AppResult<HttpResponse> {
    let slug = path.into_inner();
    let mut ctx = Context::new();

    let post = Post::by_slug(&slug)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Post '{}' not found", slug)))?;

    let content_html = post.content_html();

    ctx.insert("post", &post);
    ctx.insert("content_html", &content_html);
    ctx.insert("page_title", &format!("{} - Kadyn Pearce", post.title));

    let body = tmpl
        .render("pages/post.html", &ctx)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .insert_header(CACHE_PUBLIC_60S)
        .body(body))
}

async fn about(tmpl: web::Data<Tera>) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();

    let experiences = Experience::all().await.unwrap_or_default();
    let skills = Skill::grouped().await.unwrap_or_default();

    ctx.insert("experiences", &experiences);
    ctx.insert("skills", &skills);
    ctx.insert("page_title", "About - Kadyn Pearce");

    let body = tmpl
        .render("pages/about.html", &ctx)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .insert_header(CACHE_PUBLIC_60S)
        .body(body))
}

async fn contact(tmpl: web::Data<Tera>) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("page_title", "Contact - Kadyn Pearce");
    ctx.insert("success", &false);
    ctx.insert("error", &Option::<String>::None);

    let body = tmpl
        .render("pages/contact.html", &ctx)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn contact_submit(
    tmpl: web::Data<Tera>,
    form: web::Form<CreateMessage>,
) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("page_title", "Contact - Kadyn Pearce");

    // Basic validation
    if form.name.trim().is_empty()
        || form.email.trim().is_empty()
        || form.subject.trim().is_empty()
        || form.message.trim().is_empty()
    {
        ctx.insert("success", &false);
        ctx.insert("error", &Some("All fields are required"));
        let body = tmpl.render("pages/contact.html", &ctx)?;
        return Ok(HttpResponse::BadRequest()
            .content_type("text/html")
            .body(body));
    }

    // Simple email validation
    if !form.email.contains('@') {
        ctx.insert("success", &false);
        ctx.insert("error", &Some("Invalid email address"));
        let body = tmpl.render("pages/contact.html", &ctx)?;
        return Ok(HttpResponse::BadRequest()
            .content_type("text/html")
            .body(body));
    }

    // Save message to database
    let form_data = form.into_inner();
    let name = form_data.name.clone();
    let email = form_data.email.clone();
    let subject = form_data.subject.clone();
    let message = form_data.message.clone();

    match Message::create(form_data).await {
        Ok(_) => {
            // Send email notification to admin (don't fail if email fails)
            if let Err(e) =
                EmailService::send_contact_notification(&name, &email, &subject, &message).await
            {
                warn!("Failed to send contact notification email: {}", e);
            }

            // Send auto-reply to sender (don't fail if email fails)
            if let Err(e) = EmailService::send_contact_auto_reply(&name, &email).await {
                warn!("Failed to send auto-reply email: {}", e);
            }

            ctx.insert("success", &true);
            ctx.insert("error", &Option::<String>::None);
        }
        Err(_) => {
            ctx.insert("success", &false);
            ctx.insert("error", &Some("Failed to send message. Please try again."));
        }
    }

    let body = tmpl.render("pages/contact.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
