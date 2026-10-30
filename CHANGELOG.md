# Changelog

Every notable change of vigil. Sections are written by `env/scripts/changelog.sh` from the
Conventional Commits between release tags; the release on GitHub carries the section of its tag.
The versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Features

- **GH-43:** Updated findings, sudo actions, secrets clean ([4624bc4](https://github.com/hoknie/vigil/commit/4624bc400e59af1151f505852e43f0fda007af89))
- **GH-43:** Updated state and loops ([492be75](https://github.com/hoknie/vigil/commit/492be75030e9f0d2f00258afe0c6bdcb13712920))
- **GH-41:** Added wild mute for findings ([2d98d40](https://github.com/hoknie/vigil/commit/2d98d40297069e059e3760036a830a0666abf2b8))

### Dependencies

- Bump rustix in the cargo-minor-and-patch group (#37) ([643940b](https://github.com/hoknie/vigil/commit/643940b1c313ca898fe7bec66abf1d391459920b))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.10...HEAD)

## [1.0.10] - 2026-09-19

### Features

- **GH-7:** Added support for macos (#36) ([673e27b](https://github.com/hoknie/vigil/commit/673e27bb17014abf7de16d9f5f2400cea178bcd6))
- Added coverage ([f721296](https://github.com/hoknie/vigil/commit/f72129614309700574ad7991f7d7d825eac2b274))
- Ci update ([1712485](https://github.com/hoknie/vigil/commit/1712485ad1799483bc1eec1bf8ae0bdf3385f040))

### Dependencies

- Bump the actions group with 3 updates (#33) ([3c929ee](https://github.com/hoknie/vigil/commit/3c929ee342b160e9b18f075adc07149e77a120cb))
- Bump clap in the cargo-minor-and-patch group (#34) ([f343488](https://github.com/hoknie/vigil/commit/f3434888c60effce393a953d384cb5120ad8beda))
- Bump sha2 from 0.10.9 to 0.11.0 (#35) ([0645f9e](https://github.com/hoknie/vigil/commit/0645f9ebec613b15a75e19f688b6f4676d62c219))
- Bump debian in /env/docker in the images group (#32) ([c028c97](https://github.com/hoknie/vigil/commit/c028c978e452b8b71a089290ea550f19f51989be))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.9...v1.0.10)

## [1.0.9] - 2026-09-18

### Features

- **GH-20:** Extended findings and added live suppress (#31) ([f31b2ae](https://github.com/hoknie/vigil/commit/f31b2ae56ddc1101530772cf4fdd0f15479dd20a))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.8...v1.0.9)

## [1.0.8] - 2026-09-18

### Features

- **GH-19:** Added support for docker (#30) ([2678c33](https://github.com/hoknie/vigil/commit/2678c33e6ef2ea6466b71a0e02cb77a43ba82b25))
- Added install script ([4889031](https://github.com/hoknie/vigil/commit/4889031e0966026a2ca7489afd965607d952f891))

### Other

- Update README.md ([609efd7](https://github.com/hoknie/vigil/commit/609efd733c7defb61a6c816165556d9a2c184047))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.7...v1.0.8)

## [1.0.7] - 2026-09-17

### Fixes

- **GH-28:** Improved ui ([e00239c](https://github.com/hoknie/vigil/commit/e00239c86b2b9c3f5e755ad567c8922af278ad9e))
- **GH-16:** Update units, firewall and fs (#27) ([000916e](https://github.com/hoknie/vigil/commit/000916e0e15c7baed53b81a308c3182dabf6617f))
- **GH-16:** Update units, firewall and fs ([3361183](https://github.com/hoknie/vigil/commit/33611832e818cd41483eee31c75d3d1fb20a3e4e))
- **GH-15:** Added launches count and last run date (#26) ([bd7c7a5](https://github.com/hoknie/vigil/commit/bd7c7a510c9636d8fd7e9737192cf2e4d9e33a78))
- **GH-13:** Fmt ([3f32f27](https://github.com/hoknie/vigil/commit/3f32f27d80eca05b88af31e8b4432702274512ce))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.6...v1.0.7)

## [1.0.6] - 2026-09-14

### Features

- **GH-13:** Added accounts crud (#25) ([b5b18d3](https://github.com/hoknie/vigil/commit/b5b18d33e820b2cf42d20bba3aed9b231281d941))
- **GH-14:** Added last audit id and updated pid getter (#24) ([31122e6](https://github.com/hoknie/vigil/commit/31122e6d6fbd31306f8feab28bb6eb27d339d9dd))
- **GH-11:** Added socket control (#23) ([1c54096](https://github.com/hoknie/vigil/commit/1c54096345370a04b5bd2ebb85994b149041c4c8))
- **GH-10:** Added suppress for findings from ui (#22) ([8df922e](https://github.com/hoknie/vigil/commit/8df922e94f08ad0d4c1adeeae002abb876ba7d71))
- **GH-12:** Reorg crates (#21) ([9b649d1](https://github.com/hoknie/vigil/commit/9b649d1e67926dcee607af7bcfe7ca83d9d64183))

### Fixes

- Updated counter for audit ([f1ff188](https://github.com/hoknie/vigil/commit/f1ff1880e12b72f9a93cb12238c2999dbcd42d42))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.5...v1.0.6)

## [1.0.5] - 2026-09-12

### Features

- **GH-2:** Updated audit rule (#3) ([2dd71c3](https://github.com/hoknie/vigil/commit/2dd71c3c003be9ace4116f932f6aa2140617ffb2))
- Added ico ([0fa971e](https://github.com/hoknie/vigil/commit/0fa971ee9eacb1608265dbdd21aaf1d85f257485))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.4...v1.0.5)

## [1.0.4] - 2026-09-12

### Features

- **vig-548:** Added containers and fs collectors (#1) ([1c6f5e2](https://github.com/hoknie/vigil/commit/1c6f5e20704a0f7670326ac2654c4664ac1e56f1))
- Added more preview img ([51b6757](https://github.com/hoknie/vigil/commit/51b675720b21b6045c37b7da7c62008a9c7bcb61))
- Added screenshots to README.md ([f734f76](https://github.com/hoknie/vigil/commit/f734f760828046289022197a6b5e571551c3cadb))
- Added preview img ([fe6ed9e](https://github.com/hoknie/vigil/commit/fe6ed9ef6cb5f09d0bb19059b4b06cf9a68d859b))
- Ver bump ([cb18c4e](https://github.com/hoknie/vigil/commit/cb18c4e057ebfc312a118f3b861ea62c42853e98))
- Removed help msg ([65d8915](https://github.com/hoknie/vigil/commit/65d89157f6f3e14fa66980cccf784756533a46df))

### Other

- Create CODE_OF_CONDUCT.md ([c360e71](https://github.com/hoknie/vigil/commit/c360e71c22db39d004c304cf7004da048d3f14be))

[Full diff](https://github.com/hoknie/vigil/compare/v1.0.3...v1.0.4)

## [1.0.3] - 2026-09-11

### Features

- Added firewall collector ([3e804ce](https://github.com/hoknie/vigil/commit/3e804ceb11d1e9cabcb5e9a5e9167a4bede9abcc))
- Added release tools ([d136237](https://github.com/hoknie/vigil/commit/d1362371c458639fea067f937fddbbf62cc79a2a))
- Fixed period ([78bbf51](https://github.com/hoknie/vigil/commit/78bbf51f25ad716feca4741c76c9aaddba42fe91))
- Updated collectors ([5bb1fd1](https://github.com/hoknie/vigil/commit/5bb1fd1c3ad8fa9f07b07e5b3a14345a108af521))
- Added README ([319c935](https://github.com/hoknie/vigil/commit/319c93519c349408a078f85a6cda05c47e08973d))
- Init ([90581ab](https://github.com/hoknie/vigil/commit/90581ab76b72ab277d64146b50ecd00f5a5b6173))
- Added example config ([f05a7ee](https://github.com/hoknie/vigil/commit/f05a7ee4f69c35524cc9cfa5be1e2bdcf2989fdd))
- Updated packaging and justfile ([afc553c](https://github.com/hoknie/vigil/commit/afc553c53dcb3ac702d4725d5ddb44e6593e8be3))
- Added api contract ([99202ec](https://github.com/hoknie/vigil/commit/99202ec7983dec4c821c9ad11902a682e8d00dda))
- Added env ([670803c](https://github.com/hoknie/vigil/commit/670803ccbaaddb625d3c68b25ead95eef9eb25f4))

### Fixes

- Bump version ([fcaa83b](https://github.com/hoknie/vigil/commit/fcaa83b1965f64c6652b17c57388456af899a89c))
- Added missed file ([46ffdb1](https://github.com/hoknie/vigil/commit/46ffdb15813a049a3874332293ed889492ed0a04))
- Updated tests ([4fb7f48](https://github.com/hoknie/vigil/commit/4fb7f48ce9ae8fd056ce5f063ca822422fcd52d4))
- Cargo fmt ([a0c0120](https://github.com/hoknie/vigil/commit/a0c0120bdfea972575a9da87aa7a206fa7b0825d))
- Release workflow updated ([58c33de](https://github.com/hoknie/vigil/commit/58c33de7ee45e12abc9ed9c9e1b8c55d9a778ed3))
- Added missing manifest ([cb161e7](https://github.com/hoknie/vigil/commit/cb161e771da1841fa6fc7f7c23825b0746857518))

### Other

- Added packaging ([4724495](https://github.com/hoknie/vigil/commit/472449578c09e4df4663c56c6c01ac1ea8eb9e9b))
- Initial commit ([4122f87](https://github.com/hoknie/vigil/commit/4122f87ab9bb6af8e03109cb15ab2b4f3199d667))
