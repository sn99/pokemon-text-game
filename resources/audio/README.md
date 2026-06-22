# Audio assets

Place optional MP3 files here. The game tries these paths in order for each context:

| Context | Filenames tried |
|---------|-----------------|
| Title / menus | `resources/audio/track.mp3`, `resources/track.mp3` |
| Battle | `resources/audio/battle.mp3` then title track fallback |
| Victory | `resources/audio/victory.mp3` |
| Pokedex | `resources/audio/pokedex.mp3` |

If no device or file is available, the game runs silently.

**Commercial builds:** replace the default `resources/track.mp3` with original or properly licensed music only.
