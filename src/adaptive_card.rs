use serde::{Deserialize, Serialize};

use crate::buildkite::BuildData;

#[derive(Serialize, Deserialize, Debug)]
pub struct AdaptiveCardData {
    #[serde(rename = "type")]
    data_type: String,
    attachments: Vec<Attachment>,
}

impl From<BuildData> for AdaptiveCardData {
    fn from(data: BuildData) -> Self {
        let sender_name: &String = &data.sender_name;

        let adaptive_card_data = AdaptiveCardData {
            data_type: "message".to_string(),
            attachments: vec![Attachment {
                content_type: "application/vnd.microsoft.card.adaptive".to_string(),
                content_url: "none".to_string(),
                content: Content {
                    content_type: "AdaptiveCard".to_string(),
                    schema: "http://adaptivecards.io/schemas/adaptive-card.json".to_string(),
                    version: "1.5".to_string(),
                    body: vec![
                        BodyItem::TextBlock(TextBlock {
                            size: Some("medium".to_string()),
                            weight: "Bolder".to_string(),
                            text: format!(
                                "{} #{} {:?}",
                                data.pipeline.name, data.build.number, data.build.state
                            ),
                            color: "Good".to_string(),
                            wrap: None,
                            ..Default::default()
                        }),
                        BodyItem::ColumnSet(ColumnSet {
                            columns: vec![
                                Column {
                                    column_type: "Column".to_string(),
                                    items: vec![ColumnItem::Image(Image {
                                        style: "person".to_string(),
                                        url: data.creator_avatar,
                                        alt_text: sender_name.to_string(),
                                        size: "small".to_string(),
                                    })],
                                    width: "auto".to_string(),
                                },
                                Column {
                                    column_type: "Column".to_string(),
                                    items: vec![
                                        ColumnItem::TextBlock(TextBlock {
                                            weight: "Bolder".to_string(),
                                            text: sender_name.to_string(),
                                            wrap: Some(true),
                                            ..Default::default()
                                        }),
                                        ColumnItem::TextBlock(TextBlock {
                                            text: format!("Created {}", data.build.created_at),
                                            is_subtle: Some(true),
                                            spacing: "none".to_string(),
                                            wrap: Some(true),
                                            ..Default::default()
                                        }),
                                    ],
                                    width: "stretch".to_string(),
                                },
                            ],
                        }),
                        BodyItem::TextBlock(TextBlock {
                            text: format!("Repository: {}", data.pipeline.repository),
                            is_subtle: Some(true),
                            font_type: "Monospace".to_string(),
                            size: Some("small".to_string()),
                            wrap: Some(true),
                            ..Default::default()
                        }),
                        BodyItem::TextBlock(TextBlock {
                            text: format!("Commit: {}", data.build.commit),
                            is_subtle: Some(true),
                            font_type: "Monospace".to_string(),
                            size: Some("small".to_string()),
                            wrap: Some(true),
                            ..Default::default()
                        }),
                    ],
                    actions: vec![Action {
                        action_type: "Action.OpenUrl".to_string(),
                        title: "View in Buildkite".to_string(),
                        url: format!("{}/builds/{}", data.pipeline.web_url, data.build.number),
                    }],
                },
            }],
        };
        adaptive_card_data
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Attachment {
    #[serde(rename = "contentType")]
    content_type: String,
    #[serde(rename = "contentUrl")]
    content_url: String,
    content: Content,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    body: Vec<BodyItem>,
    actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum BodyItem {
    TextBlock(TextBlock),
    ColumnSet(ColumnSet),
}

#[derive(Serialize, Deserialize, Debug)]
struct TextBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "weight_is_normal")]
    weight: String,
    #[serde(skip_serializing_if = "color_is_blank")]
    color: String,
    text: String,
    #[serde(skip_serializing_if = "spacing_is_blank")]
    spacing: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isSubtle")]
    is_subtle: Option<bool>,
    #[serde(skip_serializing_if = "font_is_set")]
    #[serde(rename = "fontType")]
    font_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrap: Option<bool>,
}

fn weight_is_normal(weight: &String) -> bool {
    weight == "normal"
}
fn font_is_set(font_type: &String) -> bool {
    font_type == "default"
}
fn spacing_is_blank(spacing: &String) -> bool {
    spacing == ""
}
fn color_is_blank(color: &String) -> bool {
    color == ""
}

impl Default for TextBlock {
    fn default() -> Self {
        TextBlock {
            size: None,
            weight: "normal".to_string(),
            color: "".to_string(),
            text: "".to_string(),
            spacing: "".to_string(),
            is_subtle: Some(false),
            font_type: "default".to_string(),
            wrap: Some(false),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ColumnSet {
    columns: Vec<Column>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Column {
    #[serde(rename = "type")]
    column_type: String,
    items: Vec<ColumnItem>,
    width: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ColumnItem {
    Image(Image),
    TextBlock(TextBlock),
}

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    style: String,
    url: String,
    #[serde(rename = "altText")]
    alt_text: String,
    size: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Action {
    #[serde(rename = "type")]
    action_type: String,
    title: String,
    url: String,
}
