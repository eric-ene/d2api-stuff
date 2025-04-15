
use std::fs;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};
use reqwest::{Client, Method, Request, header::HeaderMap};
use serde_json::Value;
use crate::download::DownloadError::*;
use super::macros::macros::*;

const BASE: &'static str = "https://www.bungie.net";
const REST_ENDP: &'static str = "/Platform";
const D2_MANIFEST: &'static str = "/Destiny2/Manifest";

#[derive(Debug)]
#[allow(dead_code)]
pub enum DownloadError {
  IOError(std::io::Error),
  APIError(String),
  MapError(String),
  SerdeError(serde_json::Error),
  NoneError,
  OtherError(String),
}

impl From<std::io::Error> for DownloadError {
  fn from(value: std::io::Error) -> Self {
    IOError(value)
  }
}

impl From<serde_json::Error> for DownloadError {
  fn from(value: serde_json::Error) -> Self {
    SerdeError(value)
  }
}


pub async fn download_json(category: &str, path: &str, check_date: bool) -> Result<(), DownloadError> {
  // check if file is recent enough
  if let Ok(meta) = fs::metadata(format!("data/{path}.meta")) {
    if let Ok(time) = meta.modified() {
      if time > (SystemTime::now() - Duration::from_secs(60 * 60 * 24 * 7)) && check_date {
        println!("Data is <7 days old. Not downloading.");
        return Ok(());
      }
    }
  }
  
  let api_key = match std::env::var("API_KEY") {
    Ok(key) => key,
    Err(_) => {
      println!();
      return Err(APIError("BUNGIE_API_KEY not found in env :(".to_string()));
    }
  };

  let mut default_headers = HeaderMap::new();
  default_headers.append("X-API-Key", api_key.parse().unwrap());

  let client = Client::builder()
    .default_headers(default_headers)
    .build().unwrap();

  // get d2 manifest
  let res = client.execute(Request::new(
    Method::GET,
    url!("{BASE}{REST_ENDP}{D2_MANIFEST}")
  )).await
    .expect("Request failed!")
    .json::<serde_json::Value>()
    .await.unwrap();

  // get response from response
  let res_map = res.get("Response").expect("No response in response!");

  // get the path of the big json of every item
  let json_path = res_map.get("jsonWorldContentPaths")
    .ok_or(NoneError)?.get("en")
    .ok_or(NoneError)?
    .as_str()
    .ok_or(NoneError)?;

  println!("Downloading JSON...");
  
  // get the big json of every file
  let big_json = get!(client, "{BASE}{json_path}");
  let big_json = big_json.get(category).ok_or(NoneError)?;

  println!("Saving JSON...");

  // do what the function name says
  return chunk_and_write(big_json, path);
}

fn chunk_and_write(json: &Value, path: &str) -> Result<(), DownloadError> {
  let data = match json {
    Value::Object(obj) => obj,
    ty => {
      return Err(MapError(format!("json was wrong type: {}", std::any::type_name_of_val(ty))))
    }
  };
  
  let _ = fs::remove_dir_all(format!("data/{path}"));
  let _ = fs::create_dir(format!("data/{path}"));
  
  let mut count = 0;
  let mut fileno = 0;
  let mut file;
  
  file = File::open("/dev/null")?;
  
  for (_, v) in data.iter() {
    if count == 0 {
      if fileno != 0 {
        file.write("]\n".as_bytes())?;
      }
      
      file = File::options()
        .append(true)
        .create(true)
        .open(format!("data/{path}/chunk_{fileno}.json"))?;
      
      file.write("[\n".as_bytes())?;
    } else {
      file.write(",".as_bytes())?;
    }
    
    let json_entry = format!("{}", serde_json::to_string_pretty(v)?);
    
    file.write(json_entry.as_bytes())?;
    
    count += 1;
    
    if count >= 1000 {
      count = 0;
      fileno += 1;
    }
  }
  
  // fs::write(
  //   format!("data/{path}.json"),
  //   serde_json::to_string_pretty(data)?.as_str()
  // )?;
  
  fs::write(
    format!("data/{path}.meta"), 
    format!("last download: {:?}\ntotal entries: {}", chrono::Local::now(), data.len())
  )?;
  Ok(())
}