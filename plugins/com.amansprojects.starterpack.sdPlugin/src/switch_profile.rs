//! Non-spec OpenDeck-specific protocols are used in this file.

use openaction::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SwitchProfileEvent {
	event: &'static str,
	device: String,
	profile: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	direction: Option<&'static str>,
}

// Setting values that mean "step relative to the current profile". Profile names are
// file names, which can never contain NUL, so these cannot clash with a real profile.
const NEXT_PROFILE: &str = "\u{0}next";
const PREVIOUS_PROFILE: &str = "\u{0}previous";
const NEXT_IN_FOLDER: &str = "\u{0}next_in_folder";
const PREVIOUS_IN_FOLDER: &str = "\u{0}previous_in_folder";

fn switch_event(device: String, target: Option<&str>) -> SwitchProfileEvent {
	let target = target.unwrap_or("Default");
	let direction = match target {
		NEXT_PROFILE => Some("next"),
		PREVIOUS_PROFILE => Some("previous"),
		NEXT_IN_FOLDER => Some("next_in_folder"),
		PREVIOUS_IN_FOLDER => Some("previous_in_folder"),
		_ => None,
	};
	SwitchProfileEvent {
		event: "switchProfile",
		device,
		profile: if direction.is_some() {
			String::new()
		} else {
			target.to_owned()
		},
		direction,
	}
}

/// OpenDeck's config directory. OpenDeck runs plugins from `<config>/plugins/<uuid>`.
fn config_dir() -> Option<std::path::PathBuf> {
	let cwd = std::env::current_dir().ok()?;
	Some(cwd.parent()?.parent()?.to_path_buf())
}

fn profile_folder(id: &str) -> &str {
	id.split_once('/').map_or("", |(folder, _)| folder)
}

/// Mirrors OpenDeck's menu order: top-level profiles first, then folders A-Z, each group in
/// the user's dragged order (`profile_order` in `<config>/profiles/<device>.json`), and
/// profiles missing from that order after, alphabetically.
fn order_profiles(mut profiles: Vec<String>, order: &[String]) -> Vec<String> {
	profiles.sort();
	profiles.dedup();
	let rank = |id: &String| {
		order
			.iter()
			.position(|ordered| ordered == id)
			.unwrap_or(usize::MAX)
	};
	profiles.sort_by(|a, b| {
		profile_folder(a)
			.cmp(profile_folder(b))
			.then(rank(a).cmp(&rank(b)))
			.then(a.cmp(b))
	});
	profiles
}

fn saved_order(config: &std::path::Path, device: &str) -> Vec<String> {
	std::fs::read_to_string(config.join("profiles").join(format!("{device}.json")))
		.ok()
		.and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
		.and_then(|value| serde_json::from_value(value.get("profile_order")?.clone()).ok())
		.unwrap_or_default()
}

/// Lists a device's profiles the way OpenDeck does (`<config>/profiles/<device>/*.json`,
/// one folder level deep), in OpenDeck's menu order.
pub(crate) fn device_profiles(device: &str) -> Vec<String> {
	let Some(config) = config_dir() else {
		return vec![];
	};
	let root = config.join("profiles").join(device);
	let profile_id = |name: &str| name.strip_suffix(".json").map(str::to_owned);

	let mut profiles = vec![];
	for entry in std::fs::read_dir(&root).into_iter().flatten().flatten() {
		let name = entry.file_name().to_string_lossy().into_owned();
		let path = entry.path();
		if path.is_file() {
			profiles.extend(profile_id(&name));
		} else if path.is_dir() {
			for sub in std::fs::read_dir(&path).into_iter().flatten().flatten() {
				if sub.path().is_file() {
					profiles.extend(
						profile_id(&sub.file_name().to_string_lossy())
							.map(|id| format!("{name}/{id}")),
					);
				}
			}
		}
	}
	order_profiles(profiles, &saved_order(&config, device))
}

#[cfg(test)]
mod tests {
	use super::order_profiles;

	fn ids(list: &[&str]) -> Vec<String> {
		list.iter().map(|id| id.to_string()).collect()
	}

	// Must match OpenDeck's own `order_profiles` (src-tauri/src/store/profiles.rs).
	#[test]
	fn matches_opendeck_menu_order() {
		let profiles = ids(&["In Game", "Games/A", "Default", "Games/B", "Discord"]);
		assert_eq!(
			order_profiles(profiles.clone(), &[]),
			ids(&["Default", "Discord", "In Game", "Games/A", "Games/B"])
		);
		let order = ids(&["In Game", "Default", "Games/B"]);
		assert_eq!(
			order_profiles(profiles, &order),
			ids(&["In Game", "Default", "Discord", "Games/B", "Games/A"])
		);
	}
}

async fn send_profiles(instance: &Instance, device: &str) -> OpenActionResult<()> {
	instance
		.send_to_property_inspector(serde_json::json!({
			"event": "updateProfiles",
			"device": device,
			"profiles": device_profiles(device),
		}))
		.await
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SwitchProfileSettings {
	device: Option<String>,
	profile: Option<String>,
	anticlockwise: Option<String>,
	clockwise: Option<String>,
}

pub struct SwitchProfileAction;
#[async_trait]
impl Action for SwitchProfileAction {
	const UUID: &'static str = "com.amansprojects.starterpack.switchprofile";
	type Settings = SwitchProfileSettings;

	async fn key_up(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
		let device = settings
			.device
			.as_deref()
			.unwrap_or(&instance.device_id)
			.to_owned();
		send_arbitrary_json(switch_event(device, settings.profile.as_deref())).await
	}

	async fn dial_up(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		self.key_up(instance, settings).await
	}

	async fn dial_rotate(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
		ticks: i16,
		_pressed: bool,
	) -> OpenActionResult<()> {
		let profile = if ticks < 0 {
			&settings.anticlockwise
		} else {
			&settings.clockwise
		};
		let device = settings
			.device
			.as_deref()
			.unwrap_or(&instance.device_id)
			.to_owned();
		send_arbitrary_json(switch_event(device, profile.as_deref())).await
	}

	async fn property_inspector_did_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		instance
			.send_to_property_inspector(serde_json::json!({
				"event": "updateDevices",
				"devices": get_connected_devices().await.keys().collect::<Vec<_>>(),
			}))
			.await?;
		send_profiles(
			instance,
			settings.device.as_deref().unwrap_or(&instance.device_id),
		)
		.await
	}

	async fn send_to_plugin(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
		payload: &serde_json::Value,
	) -> OpenActionResult<()> {
		// The inspector asks again when its device changes.
		if payload.get("event").and_then(|event| event.as_str()) == Some("getProfiles") {
			let device = payload
				.get("device")
				.and_then(|device| device.as_str())
				.filter(|device| !device.is_empty())
				.or(settings.device.as_deref())
				.unwrap_or(&instance.device_id)
				.to_owned();
			send_profiles(instance, &device).await?;
		}
		Ok(())
	}
}

pub(crate) async fn update_devices() -> OpenActionResult<()> {
	let message = serde_json::json!({
		"event": "updateDevices",
		"devices": get_connected_devices().await.keys().collect::<Vec<_>>(),
	});
	for instance in visible_instances(SwitchProfileAction::UUID).await {
		instance.send_to_property_inspector(message.clone()).await?;
	}
	Ok(())
}
