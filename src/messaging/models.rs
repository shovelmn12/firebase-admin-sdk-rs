use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a message to be sent via FCM.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Output Only. The identifier of the message sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Arbitrary key/value payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,

    /// Basic notification template to use across all platforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<Notification>,

    /// Android specific options for messages sent through FCM connection server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android: Option<AndroidConfig>,

    /// Webpush protocol options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webpush: Option<WebpushConfig>,

    /// Apple Push Notification Service specific options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apns: Option<ApnsConfig>,

    /// Template for FCM options across all platforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_options: Option<FcmOptions>,

    /// Registration token to send a message to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Topic name to send a message to, e.g. "weather".
    /// Note: "/topics/" prefix should not be provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Condition to send a message to, e.g. "'foo' in topics && 'bar' in topics".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Basic notification template to use across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// The notification's title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The notification's body text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The URL of an image to be downloaded on the device and displayed in the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Android specific options for messages sent through FCM connection server.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidConfig {
    /// An identifier of a group of messages that can be collapsed, so that only the last message gets sent when delivery can be resumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse_key: Option<String>,

    /// Message priority. Can be "NORMAL" or "HIGH".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AndroidMessagePriority>,

    /// How long (in seconds) the message should be kept in FCM storage if the device is offline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,

    /// Package name of the application where the registration token must match in order to receive the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_package_name: Option<String>,

    /// Arbitrary key/value payload. If present, it will override google.firebase.fcm.v1.Message.data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,

    /// Notification to send to android devices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<AndroidNotification>,

    /// Options for features provided by the FCM SDK for Android.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_options: Option<AndroidFcmOptions>,

    /// If set to true, messages will be allowed to be delivered to the app while the device is in direct boot mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_boot_ok: Option<bool>,
}

/// Priority of a message to send to Android devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AndroidMessagePriority {
    /// Normal priority.
    Normal,
    /// High priority.
    High,
}

/// Notification to send to android devices.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidNotification {
    /// The notification's title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The notification's body text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The notification's icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// The notification's icon color, expressed in #rrggbb format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// The sound to play when the device receives the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,

    /// Identifier used to replace existing notifications in the notification drawer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// The action associated with a user click on the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_action: Option<String>,

    /// The key to the body string in the app's string resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_loc_key: Option<String>,

    /// Variable string values to be used in place of the format specifiers in body_loc_key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_loc_args: Option<Vec<String>>,

    /// The key to the title string in the app's string resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_key: Option<String>,

    /// Variable string values to be used in place of the format specifiers in title_loc_key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_args: Option<Vec<String>>,

    /// The notification's channel id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    /// Sets the "ticker" text, which is sent to accessibility services.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// When set to false or unset, the notification is automatically dismissed when the user clicks it in the panel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticky: Option<bool>,

    /// Set the time that the event in the notification occurred. Notifications in the panel are sorted by this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<String>, // Timestamp format

    /// Set whether or not this notification is relevant only to the current device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_only: Option<bool>,

    /// Set the relative priority for this notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_priority: Option<NotificationPriority>,

    /// If set to true, use the Android framework's default sound for the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sound: Option<bool>,

    /// If set to true, use the Android framework's default vibrate pattern for the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_vibrate_timings: Option<bool>,

    /// If set to true, use the Android framework's default LED light settings for the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_light_settings: Option<bool>,

    /// Set the vibration pattern to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibrate_timings: Option<Vec<String>>, // Duration format

    /// Set the Notification.visibility of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,

    /// Compute the count of the number of unread messages in your application's launcher icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<i32>,

    /// Settings to control the notification's LED blinking rate and color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_settings: Option<LightSettings>,

    /// The URL of an image to be displayed in the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Priority levels for a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationPriority {
    /// Priority not specified.
    PriorityUnspecified,
    /// Min priority.
    PriorityMin,
    /// Low priority.
    PriorityLow,
    /// Default priority.
    PriorityDefault,
    /// High priority.
    PriorityHigh,
    /// Max priority.
    PriorityMax,
}

/// Visibility of the notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Visibility {
    /// Visibility not specified.
    VisibilityUnspecified,
    /// Private.
    Private,
    /// Public.
    Public,
    /// Secret.
    Secret,
}

/// Settings to control the notification's LED blinking rate and color.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LightSettings {
    /// The color of the LED.
    pub color: Option<Color>,
    /// The amount of time the LED is on.
    pub light_on_duration: Option<String>,
    /// The amount of time the LED is off.
    pub light_off_duration: Option<String>,
}

/// Represents a color in the RGB color space.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    /// The amount of red in the color as a value in the interval [0, 1].
    pub red: Option<f32>,
    /// The amount of green in the color as a value in the interval [0, 1].
    pub green: Option<f32>,
    /// The amount of blue in the color as a value in the interval [0, 1].
    pub blue: Option<f32>,
    /// The fraction of this color that should be applied to the pixel.
    pub alpha: Option<f32>,
}

/// Options for features provided by the FCM SDK for Android.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidFcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics_label: Option<String>,
}

/// Webpush protocol options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpushConfig {
    /// HTTP headers defined in webpush protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Arbitrary key/value payload. If present, it will override google.firebase.fcm.v1.Message.data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,

    /// Web Notification options as a JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<serde_json::Value>, // Webpush notification is loose JSON

    /// Options for features provided by the FCM SDK for Web.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_options: Option<WebpushFcmOptions>,
}

/// Options for features provided by the FCM SDK for Web.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpushFcmOptions {
    /// The link to open when the user clicks on the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics_label: Option<String>,
}

/// Apple Push Notification Service specific options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApnsConfig {
    /// HTTP request headers defined in Apple Push Notification Service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// APNs payload as a JSON object, including both 'aps' dictionary and custom payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<ApnsPayload>,

    /// Options for features provided by the FCM SDK for iOS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_options: Option<ApnsFcmOptions>,
}

/// APNs payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApnsPayload {
    /// The aps dictionary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aps: Option<Aps>,

    /// Custom data to include in the payload.
    #[serde(flatten)]
    pub custom_data: Option<HashMap<String, serde_json::Value>>,
}

/// The aps dictionary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Aps {
    /// The alert dictionary or string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<ApsAlert>,

    /// The badge number to display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i32>,

    /// The name of a sound file to play.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,

    /// Content available flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_available: Option<i32>, // 1

    /// Mutable content flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutable_content: Option<i32>, // 1

    /// The category identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// The thread identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

/// An alert which can be a string or a dictionary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApsAlert {
    /// An alert message string.
    String(String),
    /// An alert dictionary.
    Dictionary(ApsAlertDictionary),
}

/// An alert dictionary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ApsAlertDictionary {
    /// The title of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The subtitle of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// The body text of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// The key to the body string in the app's Localizable.strings file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_key: Option<String>,
    /// Variable string values to appear in place of the format specifiers in loc_key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_args: Option<Vec<String>>,
    /// The key to the title string in the app's Localizable.strings file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_key: Option<String>,
    /// Variable string values to appear in place of the format specifiers in title_loc_key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_args: Option<Vec<String>>,
    /// The key to the subtitle string in the app's Localizable.strings file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_loc_key: Option<String>,
    /// Variable string values to appear in place of the format specifiers in subtitle_loc_key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_loc_args: Option<Vec<String>>,
    /// The key to the label of the action button.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_loc_key: Option<String>,
    /// The filename of an image file in the app bundle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_image: Option<String>,
}

/// Options for features provided by the FCM SDK for iOS.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApnsFcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics_label: Option<String>,
    /// URL of an image to be displayed in the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Template for FCM options across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics_label: Option<String>,
}

/// Response from the topic management APIs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopicManagementResponse {
    /// The number of tokens successfully subscribed/unsubscribed.
    pub success_count: usize,
    /// The number of tokens that failed to subscribe/unsubscribe.
    pub failure_count: usize,
    /// The list of errors.
    pub errors: Vec<TopicManagementError>,
}

/// Error details for a single registration token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicManagementError {
    /// The index of the token in the request list.
    pub index: usize,
    /// The error message.
    pub reason: String,
}

/// Response from a batch send operation.
#[derive(Debug, Clone, Default)]
pub struct BatchResponse {
    /// The number of messages successfully sent.
    pub success_count: usize,
    /// The number of messages that failed to send.
    pub failure_count: usize,
    /// The list of responses for each message.
    pub responses: Vec<SendResponse>,
}

/// Response for an individual message in a batch.
#[derive(Debug, Clone)]
pub struct SendResponse {
    /// Whether the message was sent successfully.
    pub success: bool,
    /// The message ID, if sent successfully.
    pub message_id: Option<String>,
    /// The error message, if failed.
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SendResponseInternal {
    pub name: String,
}
