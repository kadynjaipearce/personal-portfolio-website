use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminUser;
use crate::models::{
    CreateExperience, CreatePost, CreateProject, CreateSkill, Experience, Message, Post, Project,
    Skill, UpdateExperience, UpdatePost, UpdateProject,
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use tera::{Context, Tera};

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // Dashboard
            .route("", web::get().to(dashboard))
            .route("/", web::get().to(dashboard))
            // Projects
            .route("/projects", web::get().to(projects_list))
            .route("/projects/new", web::get().to(project_form))
            .route("/projects/new", web::post().to(project_create))
            .route("/projects/rewrite", web::post().to(project_rewrite_ai))
            .route("/projects/{id}/edit", web::get().to(project_edit))
            .route("/projects/{id}/edit", web::post().to(project_update))
            .route("/projects/{id}/delete", web::post().to(project_delete))
            // Posts
            .route("/posts", web::get().to(posts_list))
            .route("/posts/new", web::get().to(post_form))
            .route("/posts/new", web::post().to(post_create))
            .route("/posts/rewrite", web::post().to(post_rewrite_ai))
            .route("/posts/{id}/edit", web::get().to(post_edit))
            .route("/posts/{id}/edit", web::post().to(post_update))
            .route("/posts/{id}/delete", web::post().to(post_delete))
            .route("/posts/{id}/tweet", web::post().to(post_tweet))
            // Experience
            .route("/experience", web::get().to(experience_list))
            .route("/experience/new", web::get().to(experience_form))
            .route("/experience/new", web::post().to(experience_create))
            .route("/experience/{id}/edit", web::get().to(experience_edit))
            .route("/experience/{id}/edit", web::post().to(experience_update))
            .route("/experience/{id}/delete", web::post().to(experience_delete))
            // Skills
            .route("/skills", web::get().to(skills_list))
            .route("/skills/new", web::post().to(skill_create))
            .route("/skills/{id}/delete", web::post().to(skill_delete))
            // Messages
            .route("/messages", web::get().to(messages_list))
            .route("/messages/{id}", web::get().to(message_view))
            .route("/messages/{id}/delete", web::post().to(message_delete)),
    );
}

// Dashboard
async fn dashboard(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();

    let project_count = Project::count().await.unwrap_or(0);
    let post_count = Post::count().await.unwrap_or(0);
    let unread_messages = Message::count_unread().await.unwrap_or(0);
    let recent_messages = Message::recent(5).await.unwrap_or_default();

    ctx.insert("user", &user);
    ctx.insert("project_count", &project_count);
    ctx.insert("post_count", &post_count);
    ctx.insert("unread_messages", &unread_messages);
    ctx.insert(
        "recent_messages",
        &recent_messages,
    );
    ctx.insert("page_title", "Dashboard - Admin");

    let body = tmpl.render("admin/dashboard.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

// Projects CRUD
async fn projects_list(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    let projects = Project::all().await?;

    ctx.insert("user", &user);
    ctx.insert("projects", &projects);
    ctx.insert("page_title", "Projects - Admin");

    let body = tmpl.render("admin/projects/list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn project_form(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("user", &user);
    ctx.insert("project", &Option::<Project>::None);
    ctx.insert("page_title", "New Project - Admin");

    let body = tmpl.render("admin/projects/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[derive(Deserialize)]
pub struct ProjectRewriteFormData {
    pub description: String,
}

async fn project_rewrite_ai(
    _user: AdminUser,
    form: web::Form<ProjectRewriteFormData>,
) -> AppResult<HttpResponse> {
    let rewritten = crate::services::openrouter::OpenRouterService::rewrite_markdown(&form.description).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(rewritten))
}

#[derive(Deserialize)]
pub struct ProjectFormData {
    pub name: String,
    pub company: Option<String>,
    pub description: String,
    pub project_type: String,
    pub url: Option<String>,
    pub site_image_url: Option<String>,
    pub client_image_url: Option<String>,
    pub tags: String,
    pub stars: Option<String>,
    pub sort_order: Option<String>,
    pub is_featured: Option<String>,
}

async fn project_create(
    _user: AdminUser,
    form: web::Form<ProjectFormData>,
) -> AppResult<HttpResponse> {
    let tags: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let data = CreateProject {
        name: form.name.clone(),
        company: form.company.clone().filter(|s| !s.is_empty()),
        description: form.description.clone(),
        project_type: form.project_type.clone(),
        url: form.url.clone().filter(|s| !s.is_empty()),
        site_image_url: form.site_image_url.clone().filter(|s| !s.is_empty()),
        client_image_url: form.client_image_url.clone().filter(|s| !s.is_empty()),
        tags,
        stars: form.stars.clone().and_then(|s| s.parse().ok()).unwrap_or(0),
        sort_order: form.sort_order.clone().and_then(|s| s.parse().ok()).unwrap_or(0),
        is_featured: form.is_featured.is_some(),
    };

    Project::create(data).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/projects"))
        .finish())
}

async fn project_edit(
    tmpl: web::Data<Tera>,
    user: AdminUser,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let mut ctx = Context::new();

    let project = Project::by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    ctx.insert("user", &user);
    ctx.insert("project", &Some(&project));
    ctx.insert("page_title", &format!("Edit {} - Admin", project.name));

    let body = tmpl.render("admin/projects/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn project_update(
    _user: AdminUser,
    path: web::Path<String>,
    form: web::Form<ProjectFormData>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();

    let tags: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let data = UpdateProject {
        name: Some(form.name.clone()),
        company: form.company.clone(),
        description: Some(form.description.clone()),
        project_type: Some(form.project_type.clone()),
        url: form.url.clone(),
        site_image_url: form.site_image_url.clone(),
        client_image_url: form.client_image_url.clone(),
        tags: Some(tags),
        stars: form.stars.clone().and_then(|s| s.parse().ok()),
        sort_order: form.sort_order.clone().and_then(|s| s.parse().ok()),
        is_featured: Some(form.is_featured.is_some()),
    };

    Project::update(&id, data).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/projects"))
        .finish())
}

async fn project_delete(_user: AdminUser, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    Project::delete(&id).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/projects"))
        .finish())
}

// Posts CRUD
async fn posts_list(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    let posts = Post::all().await?;

    ctx.insert("user", &user);
    ctx.insert("posts", &posts);
    ctx.insert("page_title", "Posts - Admin");

    let body = tmpl.render("admin/posts/list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn post_form(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("user", &user);
    ctx.insert("post", &Option::<Post>::None);
    ctx.insert("page_title", "New Post - Admin");

    let body = tmpl.render("admin/posts/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[derive(Deserialize)]
pub struct PostFormData {
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub tags: String,
    pub published: Option<String>,
}

#[derive(Deserialize)]
pub struct RewriteFormData {
    pub content: String,
}

async fn post_rewrite_ai(
    _user: AdminUser,
    form: web::Form<RewriteFormData>,
) -> AppResult<HttpResponse> {
    let rewritten =
        crate::services::openrouter::OpenRouterService::rewrite_markdown(&form.content).await?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(rewritten))
}

async fn post_create(_user: AdminUser, form: web::Form<PostFormData>) -> AppResult<HttpResponse> {
    let tags: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let data = CreatePost {
        title: form.title.clone(),
        excerpt: form.excerpt.clone(),
        content: form.content.clone(),
        tags,
        published: form.published.is_some(),
    };

    Post::create(data).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/posts"))
        .finish())
}

async fn post_edit(
    tmpl: web::Data<Tera>,
    user: AdminUser,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let mut ctx = Context::new();

    let post = Post::by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    ctx.insert("user", &user);
    ctx.insert("post", &Some(&post));
    ctx.insert("page_title", &format!("Edit {} - Admin", post.title));

    let body = tmpl.render("admin/posts/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn post_update(
    _user: AdminUser,
    path: web::Path<String>,
    form: web::Form<PostFormData>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();

    let tags: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let data = UpdatePost {
        title: Some(form.title.clone()),
        excerpt: Some(form.excerpt.clone()),
        content: Some(form.content.clone()),
        tags: Some(tags),
        published: Some(form.published.is_some()),
    };

    Post::update(&id, data).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/posts"))
        .finish())
}

async fn post_delete(_user: AdminUser, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    Post::delete(&id).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/posts"))
        .finish())
}

async fn post_tweet(_user: AdminUser, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();

    if let Some(post) = Post::by_id(&id).await? {
        if post.published {
            if let Err(e) = crate::services::twitter::TwitterService::post_new_blog_post(&post.title, &post.slug).await {
                tracing::warn!("Twitter post failed: {}", e);
            }
        }
    }

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/posts"))
        .finish())
}

// Experience CRUD
async fn experience_list(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    let experiences = Experience::all().await?;

    ctx.insert("user", &user);
    ctx.insert("experiences", &experiences);
    ctx.insert("page_title", "Experience - Admin");

    let body = tmpl.render("admin/experience/list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn experience_form(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("user", &user);
    ctx.insert("experience", &Option::<Experience>::None);
    ctx.insert("page_title", "New Experience - Admin");

    let body = tmpl.render("admin/experience/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[derive(Deserialize)]
pub struct ExperienceFormData {
    pub company: String,
    pub role: String,
    pub description: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub current: Option<String>,
    pub order_index: i32,
}

async fn experience_create(
    _user: AdminUser,
    form: web::Form<ExperienceFormData>,
) -> AppResult<HttpResponse> {
    let data = CreateExperience {
        company: form.company.clone(),
        role: form.role.clone(),
        description: form.description.clone(),
        start_date: form.start_date.clone(),
        end_date: form.end_date.clone().filter(|s| !s.is_empty()),
        current: form.current.is_some(),
        order_index: form.order_index,
    };

    Experience::create(data).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/experience"))
        .finish())
}

async fn experience_edit(
    tmpl: web::Data<Tera>,
    user: AdminUser,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let mut ctx = Context::new();

    let experience = Experience::by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Experience not found".to_string()))?;

    ctx.insert("user", &user);
    ctx.insert("experience", &Some(&experience));
    ctx.insert("page_title", "Edit Experience - Admin");

    let body = tmpl.render("admin/experience/form.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn experience_update(
    _user: AdminUser,
    path: web::Path<String>,
    form: web::Form<ExperienceFormData>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();

    let data = UpdateExperience {
        company: Some(form.company.clone()),
        role: Some(form.role.clone()),
        description: Some(form.description.clone()),
        start_date: Some(form.start_date.clone()),
        end_date: form.end_date.clone(),
        current: Some(form.current.is_some()),
        order_index: Some(form.order_index),
    };

    Experience::update(&id, data).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/experience"))
        .finish())
}

async fn experience_delete(_user: AdminUser, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    Experience::delete(&id).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/experience"))
        .finish())
}

// Skills CRUD
async fn skills_list(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    let skills = Skill::grouped().await?;

    ctx.insert("user", &user);
    ctx.insert("skills", &skills);
    ctx.insert("page_title", "Skills - Admin");

    let body = tmpl.render("admin/skills/list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn skill_create(_user: AdminUser, form: web::Form<CreateSkill>) -> AppResult<HttpResponse> {
    Skill::create(form.into_inner()).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/skills"))
        .finish())
}

async fn skill_delete(_user: AdminUser, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    Skill::delete(&id).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/skills"))
        .finish())
}

// Messages
async fn messages_list(tmpl: web::Data<Tera>, user: AdminUser) -> AppResult<HttpResponse> {
    let mut ctx = Context::new();
    let messages = Message::all().await?;

    ctx.insert("user", &user);
    ctx.insert("messages", &messages);
    ctx.insert("page_title", "Messages - Admin");

    let body = tmpl.render("admin/messages/list.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn message_view(
    tmpl: web::Data<Tera>,
    user: AdminUser,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let mut ctx = Context::new();

    let message = Message::by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Message not found".to_string()))?;

    // Mark as read
    Message::mark_read(&id).await.ok();

    ctx.insert("user", &user);
    ctx.insert("message", &message);
    ctx.insert("page_title", "View Message - Admin");

    let body = tmpl.render("admin/messages/view.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn message_delete(_user: AdminUser, path: web::Path<String>) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    Message::delete(&id).await?;

    Ok(HttpResponse::Found()
        .insert_header(("Location", "/admin/messages"))
        .finish())
}
