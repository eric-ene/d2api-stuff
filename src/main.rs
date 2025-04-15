mod download;
mod macros;

use crate::download::{download_json, DownloadError};
// Destiny.DestinyItemType.Armor = 2
// relevant path in the api = jsonWorldContentPaths
// language = "en"
// manifest path = Destiny2/Manifest
// _GET = "GET"

// process:
// 1. GET request for Destiny2/Manifest
// 2. 


#[tokio::main]
async fn main() -> Result<(), DownloadError> {
    download_json("DestinyPlugSetDefinition", "perks", false).await?;
    download_json("DestinyInventoryItemDefinition", "items", false).await?;
    
    return Ok(());
}
