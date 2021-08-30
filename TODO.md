# TODO

## Polish

- [x] Set up destroy animation (scale out, increase transparency)
- [ ] Set up spawn animation (reverse of destroy, scale in, decrease transparency)
- [x] Add sound effects
- [ ] Add music
- [ ] Logo

## Look, interface, and state

- [ ] Add UI (`bevy_egui` seems to be the way to go)
- [ ] Add state management system (menu, game, game over)
- [ ] Add menu arena size adjustment before game start
- [x] Themes

## Gameplay

- [x] Prevent food from spawning on top of any already occupied grid position
- [x] Add respawn delay that waits for destroy animation to finish
- [x] Add maps
- [ ] Add wraparound
- [ ] Add powerups/hazards
	- [ ] Drill (triangle head, able to go through three(?) walls)
	- [ ] Cherry (reduce length, from Nibbles)
	- [ ] Diamond (reverse direction, from Nibbles)
- [ ] Portals

## Misc.

- [ ] Add segment (and head) rotation.
	This would make it possible to have non square/circular segments.
- [x] Choose a license (GPLv3)
- [x] Get Windows builds working (this was never an issue, just they don't work on a VM)

## Networking

TBD. `bevy_spicy_networking`?