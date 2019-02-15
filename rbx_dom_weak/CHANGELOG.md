# rbx\_dom\_weak Changelog

## Unreleased

## 0.3.0 (2019-02-14)
* Renamed crate from `rbx_tree` to `rbx_dom_weak`
* Added support for `Ref` values ([#8](https://github.com/LPGhatguy/rbx-dom/pull/8))
* Added `UnresolvedRbxValue` and `AmbiguousRbxValue`, intended to be used alongside `rbx_reflection` to make specifying values less verbose.

## 0.2.0 (2019-01-25)
* Added new variants for `RbxValue`:
	* Int32
	* Float32
	* Enum
	* Vector2
	* Color3
	* Color3uint8
	* Vector3int16
	* Vector2int16
	* CFrame
	* PhysicalProperties (Stub)

## 0.1.0
* Initial release
* Supports `String`, `Bool`, and `Vector3` property values