
<h1 align="center">
  <br>
  <img src="screenshot.png" alt="Markdownify">
  <br>
  Hebi 🐍
  <br>
</h1>

<h4 align="center">A work-in-progress snake clone made in Rust with the <a href="https://github.com/bevyengine/bevy">Bevy engine</a>, named after the Japanese word for snake, <ruby>蛇<rp>(</rp><rt>へび</rt><rp>)</rp></ruby>.</h4>

## Configuration

One of the things that sets apart Hebi from other snake clones is its rich configuration options. You can configure pretty much everything: the map size, tick speed, food spawn rate, window scale, etc. You can even [make your own custom color themes](#Themes) in addition to those provided! Keep in mind that Hebi is still very much a work-in-progress, so the names, behavior, and functionality of configuration options may change in the future. More configuration options including custom game maps will be coming soon.

To start configuring your game, create a `config.toml` file next to the Hebi executable.

### Example `config.toml`

```TOML
theme = "solarized_light"
grid_width = 19
grid_height = 19
corner_walls = false
tick_length = 0.15
food_ticks = 8
```

### Available configuration options

| Name                             | Type     | Default value       | Function                                                     |
| -------------------------------- | -------- | ------------------- | ------------------------------------------------------------ |
| `theme`                          | `String` | `"dracula"`         | Sets what in-game color theme Hebi should use. For example, `theme = "cavern"` would load the theme file `themes/cavern.toml`. See [Themes](#themes) for more information on creating custom themes. |
| `seed`                           | `u64`    | Random              | Sets what seed should be used for deciding snake spawn locations, food spawn locations, and food colors. Since this is an unsigned number, it must be positive and less than or equal to 18446744073709551615. |
| `grid_width`                     | `u32`    | 17                  | Sets game grid width, including any walls. For the game to run properly, it must be at least 8 if `corner_walls = false` and at least 10 if `corner_walls = true`. |
| `grid_height`                    | `u32`    | 13                  | Sets game grid height, including any walls. For the game to run properly, it must be at least 8 if `corner_walls = false` and at least 10 if `corner_walls = true`. |
| `grid_scale`                     | `u32`    | 36                  | Sets the number of pixels per grid square.                   |
| `corner_walls`                   | `bool`   | `true`              | Sets whether or not there should be 2×2 corner walls in each corner of the map to make gameplay more more interesting.<br />**⚠️ This is a temporary feature, and will be removed once map customization is added.** |
| `tick_length`                    | `f64`    | 0.2                 | Sets the amount of time between in-game ticks in seconds. For a more challenging gameplay experience, try setting this to 0.15 or 0.1. For an easier gameplay experience for those with a slow reaction time, try setting this to 0.25 or 0.3. |
| `food_ticks`                     | `u32`    | 16                  | Sets the number of ticks between each food being spawned.    |
| `snake_spawn_segments`           | `u32`    | 2                   | Sets the number of segments the snake spawns with, including the head.<br />**💥 Modifying this option from its default value may cause the game to crash.** |
| `snake_segment_despawn_interval` | `f64`    | 0.1                 | Sets the interval between each segment despawning upon snake death in seconds. You can set this to 0 if you want the entire snake to despawn at once. |
| `snake_respawn_delay`            | `f64`    | 0.5                 | Sets the delay in seconds between all segments despawning upon snake death and respawning. |
| `eat_audio`                      | `String` | `"eat.mp3"`         | Sets the audio file for the destruction of the snake's head and segments. For example, `eat_audio = "munch.mp3"` (fictional file) would load the audio file `assets/sounds/munch.mp3`. |
| `destroy_audio`                  | `String` | `"destroy.mp3"`     | Sets the audio file for the destruction of the snake's head and segments. |
| `spawn_food_audio`               | `String` | `"spawn_food.mp3"`  | Sets the audio file for when food is spawned.                |
| `spawn_snake_audio`              | `String` | `"spawn_snake.mp3"` | Sets the audio file for when the snake is spawned.           |

## Themes

Along with the provided themes `dracula`, `solarized_dark`, `solarized_light`, and `cavern`, you can create your own themes! To get started, create a new file `my_theme.toml` in the themes folder, and open it in a text editor. To start off with, you can copy in the theme settings for `cavern`:

```TOML
walls = "222233"
background = "000011"
snake = "ddddee"
food = ["5599ff", "ffc455", "ff6f55"]
```

To change a color, replace any of the hex codes provided, and make sure to remove the leading `#`. For `food`, you can put in as many colors as you like. If you decide to only have one food color, make sure to wrap it on square brackets: `food = ["5599ff"]`.

To actually use your theme, see [Configuration](#Configuration) and add `theme` to your configuration file. If the theme can't file can't be found, the game will run, but as the colors are missing everything will be magenta. Likewise, if a color is missing or incorrectly formatted in your theme file, it will also show as magenta in-game.
