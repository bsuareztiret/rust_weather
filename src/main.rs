use chrono::Duration;
use chrono::Local;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::future::Future;
use structopt::StructOpt;

#[derive(Deserialize, Serialize, Debug)]
pub struct ForecastTommorow {
    cod: String,
    message: i32,
    cnt: i32,
    list: Box<[MultipleForecast]>,
    city: City,
    // country: String,
    // population: i32,
    // timezone: i32,
    // sunrise: i64,
    // sunset: i64,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct MultipleForecast {
    weather: Box<[Weather; 1]>,
    main: MultipleMain,
    visibility: i32,
    wind: Wind,
    clouds: Clouds,
    dt: i32,
    sys: Sys2,
    pop: f64,
    // rain: Rain,
    dt_txt: String,
    // _type: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Forecast {
    coord: Coord,
    weather: Box<[Weather; 1]>,
    base: String,
    main: Temps,
    visibility: i32,
    wind: Wind,
    clouds: Clouds,
    dt: i32,
    sys: Sys,
    timezone: i32,
    id: i32,
    name: String,
    cod: i32,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Coord {
    lon: f64,
    lat: f64,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Rain {
    _3h: f64,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct City {
    id: i32,
    name: String,
    coord: Coord,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Weather {
    id: i32,
    main: String,
    description: String,
    icon: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Temps {
    temp: f64,
    feels_like: f64,
    temp_min: f64,
    temp_max: f64,
    pressure: i32,
    humidity: i32,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct MultipleMain {
    temp: f64,
    feels_like: f64,
    temp_min: f64,
    temp_max: f64,
    pressure: i32,
    humidity: i32,
    sea_level: i32,
    // grd_level: i32,
    temp_kf: f64,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Wind {
    speed: f64,
    deg: i32,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Clouds {
    all: i32,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Sys2 {
    // r#type: f64,
    pod: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Sys {
    r#type: f64,
    id: i32,
    country: String,
    sunrise: i32,
    sunset: i32,
}

#[derive(StructOpt)]
pub struct Cli {
    pub path_files: String,
    pub city_name: String,
    pub favorite: String,
    pub tommorow: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonFile {
    pub list_city: Box<[CityObject; 10]>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct CityObject {
    pub city: String,
    pub favorite: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::from_args();

    open_deserialized_loop(args).await;
}

fn is_bool(value: bool) -> bool {
    match value {
        true => true,
        false => false,
    }
}

fn is_all_city(city: String) -> bool {
    return "all".to_string().eq(&city);
}

fn comp_city(array_city: String, city: String) -> bool {
    return array_city.eq(&city);
}

fn open_deserialized_loop(args: Cli) -> impl Future<Output = usize> {
    async {
        let byte =
            fs::read_to_string(args.path_files).expect("Should have been able to read the file");
        let deserialized: JsonFile = serde_json::from_str(&byte).unwrap();
        let fav = move || args.favorite.eq("true");
        let cit_n = move || args.city_name.to_string();
        let days = move || args.tommorow.eq("true");

        let mut i = 0;
        loop {
            if i > 9 {
                break;
            }
            if is_bool(fav()) == true && deserialized.list_city[i].favorite == true {
                weather_call(&deserialized.list_city[i].city, is_bool(days())).await;
            } else if is_bool(fav()) == false && is_all_city(cit_n()) == true {
                weather_call(&deserialized.list_city[i].city, is_bool(days())).await;
            } else if is_bool(fav()) == false
                && is_all_city(cit_n()) == false
                && comp_city(deserialized.list_city[i].city.to_string(), cit_n()) == true
            {
                weather_call(&deserialized.list_city[i].city, is_bool(days())).await;
            }
            i += 1;
        }
        0
    }
}

fn is_url_one_day(str: String) -> bool {
    let compair: String = str.chars().skip(40).take(9).collect();
    if compair.eq("forecast?") {
        return true;
    } else {
        return false;
    }
}

fn kelvin_to_celsius(temp: f64) -> f64 {
    return temp - 273.15;
}

fn weather_call(city: &String, days: bool) -> impl Future<Output = usize> {
    let url: std::string::String;
    match days {
        true => {
            url = format!(
                "https://api.openweathermap.org/data/2.5/forecast?q={},BE&appid={}",
                city, "60078a5c5415c46254afa30390ba6eb6"
            )
        }
        false => {
            url = format!(
                "https://api.openweathermap.org/data/2.5/weather?q={},BE&appid={}",
                city, "60078a5c5415c46254afa30390ba6eb6"
            )
        }
    }
    async {
        if is_url_one_day(url.clone()) == false {
            let response = reqwest::get(url).await.unwrap().json::<Forecast>().await;
            match response {
                Ok(v) => println!(
                    "Today in {}, the temperature is: {:.2}",
                    v.name,
                    kelvin_to_celsius(v.main.temp)
                ),
                Err(e) => println!("error parsing header: {e:?}"),
            }
        } else {
            let response = reqwest::get(url)
                .await
                .unwrap()
                .json::<ForecastTommorow>()
                .await;
            match response {
                Ok(v) => print_today_tommorow(v),
                Err(e) => println!("error parsing header: {e:?}"),
            }
        }
        0
    }
}

fn print_today_tommorow(response: ForecastTommorow) {
    let dt = Local::now();

    let sub: String = dt.to_string().chars().take(10).collect();
    let ndt = dt + Duration::days(1);
    let adt = dt + Duration::days(2);

    let subdt: String = ndt.to_string().chars().take(10).collect();
    let subadt: String = adt.to_string().chars().take(10).collect();

    let mut i = 0;
    let mut one_date: bool = false;
    loop {
        if i >= response.list.len() {
            break;
        }
        let find_date: String = response.list[i]
            .dt_txt
            .to_string()
            .chars()
            .take(10)
            .collect();
        if one_date == false && find_date.eq(&sub) == true {
            one_date = true;
            println!(
                "{}, in the date of {}, the temperature is: {:.2}",
                response.city.name,
                sub,
                kelvin_to_celsius(response.list[i].main.temp)
            )
        } else if format!("{} 12:00:00", subdt).eq(&response.list[i].dt_txt) == true {
            println!(
                "{}, in the date of {}, the temperature is: {:.2}",
                response.city.name,
                subdt,
                kelvin_to_celsius(response.list[i].main.temp)
            );
        } else if format!("{} 12:00:00", subadt).eq(&response.list[i].dt_txt) == true {
            println!(
                "{}, in the date of {}, the temperature is: {:.2}",
                response.city.name,
                subadt,
                kelvin_to_celsius(response.list[i].main.temp)
            );
        }
        i += 1;
    }
}
