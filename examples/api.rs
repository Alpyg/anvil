use anvil_core::Svc;
use anvil_macros::*;
use axum::response::IntoResponse;
use std::sync::Arc;

pub trait Service1: std::fmt::Debug + Send + Sync + 'static {
    fn method1(&self) -> Result<&'static str, String>;
}
pub trait Service2: std::fmt::Debug + Send + Sync + 'static {
    fn method2(&self) -> Result<&'static str, String>;
}

#[derive(Debug, Clone)]
pub struct ConcreteService1 {}
impl Service1 for ConcreteService1 {
    fn method1(&self) -> Result<&'static str, String> {
        Ok("Service 1")
    }
}

#[derive(Debug, Clone)]
pub struct ConcreteService2 {}
impl Service2 for ConcreteService2 {
    fn method2(&self) -> Result<&'static str, String> {
        Ok("Service 2")
    }
}

#[app_state]
#[derive(Debug, Clone)]
pub struct AppState {
    pub service1: Arc<dyn Service1>,
    pub service2: Arc<dyn Service2>,
}

#[api(get("/endpoint0"))]
async fn endpoint0() -> impl IntoResponse {
    "No service".into_response()
}

#[api(get("/endpoint1"))]
async fn endpoint1(Svc(service1): Svc<Arc<dyn Service1>>) -> impl IntoResponse {
    service1.method1().into_response()
}

#[api(get("/endpoint2"))]
async fn endpoint2(
    Svc(service1): Svc<Arc<dyn Service1>>,
    Svc(service2): Svc<Arc<dyn Service2>>,
) -> impl IntoResponse {
    format!(
        "{} {}",
        service1.method1().unwrap(),
        service2.method2().unwrap(),
    )
    .into_response()
}

#[cron("* * * * * *", timezone = "Utc")]
async fn cronjob1(service1: Arc<dyn Service1>) {
    tracing::info!("cron {}", service1.method1().unwrap());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy()
        .add_directive("sqlx=error".parse()?)
        .add_directive("anvil=debug".parse()?);

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let state = AppState {
        service1: Arc::new(ConcreteService1 {}),
        service2: Arc::new(ConcreteService2 {}),
    };

    let services = state.services();
    let _scheduler = anvil_core::start_cron(&services).await.unwrap();

    let app = state.inject(anvil_core::build_router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
