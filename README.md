# WuWa Ploom 120 FPS Unlock
WuWa Ploom is a Rust application that allows you to unlock the option for 120 FPS (Frames Per Second) limit in Wuthering Waves. This little app provides a GUI (Graphical User Interface) to locate the game's configuration file and set the FPS limit to 120.

## Support
If you find this tool useful, you can support me on [ko-fi](https://ko-fi.com/abellio).

## How it works
The application uses the `rusqlite` library to interact with the SQLite database file that stores the game's settings. It reads the current FPS setting from the database, and if it's not already set to 120, it updates the setting to 120.

## Build the app or grab a release
Run `cargo build --release`

or grab a release, Windows Defender might find it suspicious as always.

## How to use
1. Start the app.
2. With WuWa open check and set your FPS limit to 60, then close your game.
3. You can either click on Locate or browse and find your LocalStorage.db file.
4. Launch and enjoy 120 FPS
5. Do not touch FPS or VSync options in-game.

## Bugs
The 120FPS option was supposedly removed from games official release due to bugs, so if you find any it's on you.

## Source code
The source code for this application is available in this repository.

## Disclaimer
Use this tool at your own risk. I'm not responsible for any issues that may arise from using this tool. Always make a backup of your game's configuration file before making any changes.
