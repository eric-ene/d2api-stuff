
pub mod macros {
  macro_rules! url {
    ($($arg:tt)*) => {
      reqwest::Url::parse(format!($($arg)*).as_str()).unwrap()
    };
  }
  macro_rules! get {
    ($client: expr, $($arg:tt)*) => {
      $client.execute(Request::new(
      Method::GET, 
      url!($($arg)*)
    )).await
    .expect("Request failed!")
    .json::<serde_json::Value>()
    .await.unwrap()
    };
  }
  
  pub(crate) use url;
  pub(crate) use get;
}