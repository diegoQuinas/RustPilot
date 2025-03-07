mod steps;

use std::{self, collections::HashMap};

use appium_client::{capabilities::{self, android::AndroidCapabilities}, find::By, ClientBuilder};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spinners::Spinner;

use appium_client::capabilities::{AppCapable, AppiumCapability};
use steps::execute_android_steps;

use crate::{common::tags::*, common::*};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidStep {
    AndroidNormalStep(AndroidNormalStep),
    AndroidStepFile { step_file: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidNormalStep {
    AndroidElementStep {
        selector: AndroidElementSelector,
        actions: Vec<AndroidAction>,
    },
    ScreenshotStep {
        take_screenshot: String,
    },
    LogStep {
        log: String,
    },
    Pause {
        pause: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AndroidAction {
    AssertVisible,
    TapOn,
    ScrollUntilVisible,
    InsertData { data: String },
    Pause(u64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AndroidElementSelector {
    Text {
        text: String,
    },
    Xpath {
        xpath: String,
    },
    ClassName {
        class_name: String,
        instance: Option<i32>,
    },
    Id {
        id: String,
    },
    IdWithIndex {
        id: String,
        index: u32,
    },
    Description {
        description: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StepFile {
    steps: Vec<AndroidStep>,
}

pub fn set_custom_capabilities_android(
    caps: &mut AndroidCapabilities,
    custom_caps: Vec<CustomCapability>,
) {
    for custom_capability in custom_caps.clone() {
        match custom_capability.value {
            CustomCapabilityValue::BooleanValue(value) => {
                caps.set_bool(&custom_capability.key, value)
            }
            CustomCapabilityValue::StringValue(value) => {
                caps.set_str(&custom_capability.key, &value)
            }
        }
    }
}

pub fn get_android_element_by(selector: AndroidElementSelector) -> By {
    match selector {
        AndroidElementSelector::Xpath { xpath } => By::xpath(&xpath),
        AndroidElementSelector::Text { text } => {
            By::uiautomator(&format!("new UiSelector().textMatches(\"{}\");", text))
        }
        AndroidElementSelector::Description { description } => By::uiautomator(&format!(
            "new UiSelector().descriptionMatches(\"{}\");",
            description
        )),
        AndroidElementSelector::IdWithIndex { id, index } => By::uiautomator(&format!(
            "new UiSelector().resourceIdMatches(\"{}\").index({});",
            id, index
        )),
        AndroidElementSelector::Id { id } => {
            By::uiautomator(&format!("new UiSelector().resourceIdMatches(\"{}\");", id))
        }
        AndroidElementSelector::ClassName {
            class_name,
            instance,
        } => {
            if let Some(instance) = instance {
                By::uiautomator(&format!(
                    "new UiSelector().className({}).instance({})",
                    class_name, instance
                ))
            } else {
                By::uiautomator(&format!("new UiSelector().className({})", class_name))
            }
        }
    }
}

pub async fn launch_android_main(
    capabilities: &HashMap<String, Value>,
    steps: Vec<Step>,
) -> Result<(u32, String), Box<dyn std::error::Error>> {
    // Configure the Appium driver
    let mut caps = AndroidCapabilities::new_uiautomator();

    let app_path = capabilities.get("appium:app").expect("No app path found").as_str().unwrap();
    caps.app(&app_path);

    caps.platform_version(&capabilities.get("platformVersion").expect("No platform version found").as_str().unwrap());

    for (key,value) in capabilities.iter() {
        match key.as_str() {
            "app" | "platformVersion" => continue,
            _ => {
                match value {
                    Value::String(value) => {
                        caps.set_str(key, value);
                    }
                    Value::Bool(value) => {
                        caps.set_bool(key, *value);
                    }
                    _ => {
                        eprintln!("{} Invalid value for key: {}", error_tag(), key);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
        
    

    println!("{} App path: {}", info_tag(), &capabilities.get("appium:app").unwrap().to_string().blue());
    let mut spinner = Spinner::new(
        spinners::Spinners::Arrow,
        "Launching android app".to_string(),
    );
    let client = ClientBuilder::native(caps)
        .connect("http://localhost:4723/")
        .await
        .unwrap_or_else(|e| {
            spinner.stop_with_symbol(&format!(
                "{} Failed to connect to Appium: {}",
                error_tag(),
                e
            ));
            std::process::exit(1);
        });
    spinner.stop_with_symbol("[LAUNCHED]");

    let (steps_count, report) = execute_android_steps(&client, steps).await;
    Ok((steps_count, report))
}
