use serde::{Deserialize, Serialize};
// Convert data to correct json blob.  Example from DMS:
// {
//   "type": "snitch.reporting",
//   "timestamp": "2024-04-30T05:25:37.166Z",
//   "data": {
//     "snitch": {
//       "token": "c2354d53d2",
//       "name": "Critical System Reports",
//       "notes": "Useful notes for dealing with the situation",
//       "tags": [ "critical", "reports" ],
//       "status": "healthy",
//       "previous_status": "missing"
//     }
//   }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct DmsData {
    #[serde(rename = "type")]
    pub snitch_type: SnitchType,
    pub timestamp: String,
    pub data: SnitchData,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SnitchData {
    pub snitch: SnitchSubData,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SnitchSubData {
    pub token: String,
    pub name: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub status: String,
    pub previous_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SnitchType {
    #[serde(rename = "snitch.missing")]
    Missing,
    #[serde(rename = "snitch.reporting")]
    Reporting,
    #[serde(rename = "snitch.paused")]
    Paused,
}
