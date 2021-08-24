pub mod dracula {
	use crate::colors;

	pub const BACKGROUND: &str = colors::dracula::BACKGROUND;
	pub const GRID_BACKGROUND: &str = colors::dracula::SELECTION;
	pub const SNAKE: &str = colors::dracula::FOREGROUND;
	pub const FOOD: &'static [&'static str] = &[
		colors::dracula::CYAN,
		colors::dracula::GREEN,
		colors::dracula::ORANGE,
		colors::dracula::PINK,
		colors::dracula::PURPLE,
		colors::dracula::RED,
		colors::dracula::YELLOW,
	];
}