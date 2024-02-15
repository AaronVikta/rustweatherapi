use axum::response::IntoResponse;
use axum::{http::StatusCode, routing::get, Router};
use axum::extract::Query;
use anyhow::Context;
use askama_axum::Template;

// make our own error that wraps anyhow::Error
struct AppError(anyhow::Error);

// Tell axum how to convert AppError into a response
impl IntoResponse for AppError{
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
        .into_response()
    }
}
impl <E> From <E> for AppError
where
E: Into<anyhow::Error>,{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}



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
#[derive(Template)]
#[template(path="index.html")]
struct IndexTemplate;

async fn index()-> IndexTemplate{
    IndexTemplate
}

#[derive(Deserialize)]
pub struct WeatherQuery{
    pub city:String,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse{
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub hourly: Hourly,
}

#[derive(Deserialize, Debug)]
pub struct Hourly{
    pub time: Vec<String>,
    pub temperature_2m:Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Forecast{
    pub date:String,
    pub temperature: String,
}

#[derive(Template, Deserialize, Debug)]
#[template(path="weather.html")]
pub struct WeatherDisplay{
    pub city:String,
    pub forecasts: Vec<Forecast>,
}
async fn weather(Query(params): Query<WeatherQuery>) -> Result<WeatherDisplay, AppError> {
	let lat_long = fetch_lat_long(&params.city).await?;
    let weather = fetch_weather(lat_long).await?;

   Ok(WeatherDisplay::new(params.city, weather))
}

async fn stats()-> &'static str{
    "Stats"
}

impl WeatherDisplay {
    // create a new weatherDisplay from a WeatherResponse
    fn new(city:String, response:WeatherResponse)->Self{
        let display= WeatherDisplay{
            city,
            forecasts:response
            .hourly
            .time
            .iter()
            .zip(response.hourly.temperature_2m.iter())
            .map(|(date, temperature)|Forecast{
                date: date.to_string(),
                temperature: temperature.to_string(),
            })
            .collect(),
        };
        display
    }
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, anyhow::Error> {
	let endpoint = format!(
    	"https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
    	city
	);
	let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
	response.results.get(0).cloned().context("No results found")
}

async fn fetch_weather(lat_long:LatLong) ->Result<WeatherResponse, anyhow::Error >{
    let endpoint = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&endpoint).await?.json::<WeatherResponse>().await?;
    Ok(response)
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

