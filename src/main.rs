use axum::{http::StatusCode, routing::get, Router};
use axum::extract::Query;



// Struct that holds the latitude and longitude
use serde::Deserialize;
#[derive(Deserialize)]
pub struct GeoResponse{
    pub results: Vec<LatLong>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct LatLong{
    pub latitude: f64,
    pub longitude:f64,
}

// basic handler that responds with a static string
async fn index()-> &'static str{
    "Index"
}

#[derive(Deserialize)]
pub struct WeatherQuery{
    pub city:String,
}

async fn weather(Query(params): Query<WeatherQuery>) -> Result<String, StatusCode> {
	let lat_long = fetch_lat_long(&params.city)
    	.await
    	.map_err(|_| StatusCode::NOT_FOUND)?;
	Ok(format!(
    	"{}: {}, {}",
    	params.city, lat_long.latitude, lat_long.longitude
	))
}

async fn stats()-> &'static str{
    "Stats"
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
	let endpoint = format!(
    	"https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
    	city
	);
	let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
	response
    	.results
    	.get(0)
    	.cloned()
    	.ok_or("No results found".into())
}
#[tokio::main]
async fn main(){
    let app = Router::new()
    .route("/", get(index))
    .route("/weather", get(weather))
    .route("/stats", get(stats));

    let addr = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(addr, app).await.unwrap();
}

