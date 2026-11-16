//! Non-spec OpenDeck-specific details are used in this file: instance IDs have the form
//! `device.profile.controller.position.index`, and profiles are read from OpenDeck's config.

use openaction::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Display {
	/// The profile's name, without any folder prefix.
	#[default]
	Name,
	/// The profile's position in the device's sorted profile list, from 1.
	Number,
	/// Position and total, e.g. "2/5".
	NumberOfTotal,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PageIndicatorSettings {
	display: Display,
}

/// The profile an instance lives in, from its ID. A profile name may contain dots, so the
/// profile is everything between the device (first segment) and the last three segments.
fn instance_profile(instance_id: &str) -> Option<String> {
	let segments: Vec<&str> = instance_id.split('.').collect();
	if segments.len() < 5 {
		return None;
	}
	Some(segments[1..segments.len() - 3].join("."))
}

fn indicator_text(display: Display, profile: &str, profiles: &[String]) -> String {
	let number = profiles
		.iter()
		.position(|candidate| candidate == profile)
		.map(|index| index + 1);
	match (display, number) {
		(Display::Name, _) | (_, None) => profile.rsplit('/').next().unwrap_or(profile).to_owned(),
		(Display::Number, Some(number)) => number.to_string(),
		(Display::NumberOfTotal, Some(number)) => format!("{number}/{}", profiles.len()),
	}
}

async fn show(instance: &Instance, settings: &PageIndicatorSettings) -> OpenActionResult<()> {
	let Some(profile) = instance_profile(&instance.instance_id) else {
		return Ok(());
	};
	let profiles = crate::switch_profile::device_profiles(&instance.device_id);
	instance
		.set_title(
			Some(indicator_text(settings.display, &profile, &profiles)),
			None,
		)
		.await
}

pub struct PageIndicatorAction;
#[async_trait]
impl Action for PageIndicatorAction {
	const UUID: &'static str = "com.amansprojects.starterpack.pageindicator";
	type Settings = PageIndicatorSettings;

	async fn will_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		show(instance, settings).await
	}

	async fn did_receive_settings(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		show(instance, settings).await
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn profiles() -> Vec<String> {
		["Default", "Discord", "Games/Racing", "In Game"]
			.map(String::from)
			.to_vec()
	}

	#[test]
	fn reads_the_profile_from_the_instance_id() {
		assert_eq!(
			instance_profile("18-4250D278254B.In Game.Keypad.3.0").as_deref(),
			Some("In Game")
		);
		assert_eq!(
			instance_profile("sd-ABC.v1.2.Keypad.0.0").as_deref(),
			Some("v1.2")
		);
		assert_eq!(instance_profile("bad.id"), None);
	}

	#[test]
	fn shows_name_number_or_number_of_total() {
		assert_eq!(
			indicator_text(Display::Name, "Games/Racing", &profiles()),
			"Racing"
		);
		assert_eq!(indicator_text(Display::Number, "Discord", &profiles()), "2");
		assert_eq!(
			indicator_text(Display::NumberOfTotal, "In Game", &profiles()),
			"4/4"
		);
		// Not on disk yet (unsaved): fall back to the name rather than a wrong number.
		assert_eq!(indicator_text(Display::Number, "New", &profiles()), "New");
	}
}
