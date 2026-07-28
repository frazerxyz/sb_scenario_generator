# Sweatbox scenario generator

![Screenshot of the scenario generator in action](/.github/media/example.png)

## How to run
1. Download the latest release [**here**](https://github.com/frazerxyz/sb_scenario_generator/releases/latest)
2. Unzip the folder, making sure the .exe lives alongside the data folder
3. Double click the .exe file to run
```
sb_scenario_generator/
├─ sb_scenario_generator.exe
├─ data/
│  ├─ airports/
│  │  ├─ EGKK.json
│  │  ├─ ...
│  ├─ aircraft_perf.txt
│  ├─ global.json
│  ├─ ukcp_aircraft.json
```

### Windows SmartScreen
You will most likely encounter a "Windows protected your PC" error when running the exe.  
Click **More info** -> **Run anyway** to bypass.

## Custom airport files
Airport files containing an update URL like the example below will be automatically overwritten by the generator.
```json
{
    "icao": "EGKK",
    "update_url": "https://raw.githubusercontent.com/...",
    "elevation": 203.2,
    ...
}
```
If you want to make your own changes, it's recommended to duplicate the file you want to modfy, give it a different name, and **remove the update URL**.

If you feel like your changes could benefit other users, consider contributing via a PR.

## Disclaimer
I am new to to coding and Rust, please be kind!

**This is a WIP. If you find any bugs, please let me know by opening an issue.**