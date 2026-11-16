use super::{ContextEvent, PayloadEvent};

use tauri::{Emitter, Manager};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OpenUrlEvent {
	pub url: String,
}

pub async fn open_url(event: PayloadEvent<OpenUrlEvent>) -> Result<(), anyhow::Error> {
	log::debug!("Opening URL {}", event.payload.url);
	open::that_detached(event.payload.url)?;
	Ok(())
}

#[derive(Deserialize)]
pub struct LogMessageEvent {
	pub message: String,
}

pub async fn log_message(uuid: Option<&str>, mut event: PayloadEvent<LogMessageEvent>) -> Result<(), anyhow::Error> {
	if let Some(uuid) = uuid
		&& let Ok(manifest) = crate::plugins::manifest::read_manifest(&crate::shared::config_dir().join("plugins").join(uuid))
	{
		event.payload.message = format!("[{}] {}", manifest.name, event.payload.message);
	}
	log::info!("{}", event.payload.message.trim());
	Ok(())
}

pub async fn show_alert(event: ContextEvent) -> Result<(), anyhow::Error> {
	let app = crate::APP_HANDLE.get().unwrap();
	app.get_webview_window("main").unwrap().emit("show_alert", event.context)?;
	Ok(())
}

pub async fn show_ok(event: ContextEvent) -> Result<(), anyhow::Error> {
	let app = crate::APP_HANDLE.get().unwrap();
	app.get_webview_window("main").unwrap().emit("show_ok", event.context)?;
	Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SwitchProfileEvent {
	device: String,
	#[serde(default)]
	profile: String,
	/// Optional (OpenDeck-specific): switches relative to the device's current profile, in menu
	/// order and wrapping around; `profile` is then ignored. "next" / "previous" step through all
	/// profiles, "next_in_folder" / "previous_in_folder" only through the current profile's folder.
	#[serde(default, skip_serializing)]
	direction: Option<String>,
}

/// Picks the profile after (or before) `current` in `profiles` (already in menu order),
/// wrapping around. With `in_folder`, only profiles in `current`'s folder are considered.
fn step_profile(profiles: Vec<String>, current: &str, forward: bool, in_folder: bool) -> Option<String> {
	let folder = crate::store::profiles::profile_folder(current);
	let profiles: Vec<String> = profiles.into_iter().filter(|profile| !in_folder || crate::store::profiles::profile_folder(profile) == folder).collect();
	if profiles.is_empty() {
		return None;
	}
	let len = profiles.len();
	let index = match profiles.iter().position(|profile| profile == current) {
		Some(index) if forward => (index + 1) % len,
		Some(index) => (index + len - 1) % len,
		// The current profile is not on disk (e.g. not saved yet): start at an end.
		None if forward => 0,
		None => len - 1,
	};
	profiles.into_iter().nth(index)
}

pub async fn switch_profile(mut event: SwitchProfileEvent) -> Result<(), anyhow::Error> {
	if let Some(direction) = event.direction.as_deref() {
		let (forward, in_folder) = match direction {
			"next" => (true, false),
			"previous" => (false, false),
			"next_in_folder" => (true, true),
			"previous_in_folder" => (false, true),
			other => anyhow::bail!("unknown profile direction {other}"),
		};
		let (profiles, current) = {
			let mut stores = crate::store::profiles::DEVICE_STORES.write().await;
			(stores.ordered_device_profiles(&event.device)?, stores.get_selected_profile(&event.device)?)
		};
		match step_profile(profiles, &current, forward, in_folder) {
			Some(profile) => event.profile = profile,
			None => return Ok(()),
		}
	}
	let app_handle = crate::APP_HANDLE.get().unwrap();
	app_handle.get_webview_window("main").unwrap().emit("switch_profile", event)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::step_profile;

	// Already in menu order, as `ordered_device_profiles` returns them.
	fn profiles() -> Vec<String> {
		["Default", "Discord", "Games/Racing", "Games/Shooter"].map(String::from).to_vec()
	}

	#[test]
	fn steps_through_all_profiles_and_wraps() {
		assert_eq!(step_profile(profiles(), "Discord", true, false).as_deref(), Some("Games/Racing"));
		assert_eq!(step_profile(profiles(), "Games/Shooter", true, false).as_deref(), Some("Default"));
		assert_eq!(step_profile(profiles(), "Default", false, false).as_deref(), Some("Games/Shooter"));
	}

	#[test]
	fn in_folder_stays_within_the_current_folder() {
		assert_eq!(step_profile(profiles(), "Games/Shooter", true, true).as_deref(), Some("Games/Racing"));
		assert_eq!(step_profile(profiles(), "Default", true, true).as_deref(), Some("Discord"));
		assert_eq!(step_profile(profiles(), "Discord", true, true).as_deref(), Some("Default"));
	}

	#[test]
	fn unknown_current_profile_starts_at_an_end() {
		assert_eq!(step_profile(profiles(), "Gone", true, false).as_deref(), Some("Default"));
		assert_eq!(step_profile(profiles(), "Gone", false, false).as_deref(), Some("Games/Shooter"));
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceBrightnessEvent {
	action: String,
	value: u8,
}

pub async fn device_brightness(event: DeviceBrightnessEvent) -> Result<(), anyhow::Error> {
	let app_handle = crate::APP_HANDLE.get().unwrap();
	app_handle.get_webview_window("main").unwrap().emit("device_brightness", event)?;
	Ok(())
}
