# 🌍 GeoData Setup

*Enable advanced pathfinding and world simulation*

---

## What is GeoData?

GeoData files contain terrain height information and pathfinding data for the Game world. L2R uses L2J-compatible geodata files to provide accurate movement validation, collision detection, and A* pathfinding algorithms.

## Download GeoData Files

To enable advanced pathfinding and world simulation, you need L2J geodata files. Download the recommended geodata pack below:

| Source | Link | Size | Notes |
|--------|------|------|-------|
| 🔄 **Yandex Disk** | [geo.zip](https://disk.yandex.by/d/1y5yYqo7hyKxPg) | ~200MB | HF geodata |
| 📁 **Google Drive** | [geo.zip](https://drive.google.com/file/d/1L_6yH3PVedKGP9u7Y3nKU-o8rRTXKgKA/view?usp=sharing) | ~200MB | HF geodata |

## 📋 **Installation Instructions**

1. **Download** `geo.zip` from any mirror above
2. **Extract** to `game_server/data/geo/`
3. **Verify** the folder contains `.l2j` geodata files (e.g., `25_24.l2j`, `25_25.l2j`, ...)
4. **Automatic Loading** - Geodata will be loaded automatically when a region is accessed by an online player

## 📁 **File Structure**

After extraction, your geodata directory should look like:
```
game_server/data/geo/
├── 16_13.l2j
├── 16_14.l2j
├── 17_13.l2j
├── 25_24.l2j
├── 25_25.l2j
└── ... (many more .l2j files)
```

## 🎯 **What GeoData Enables**

- **🗺️ Accurate Pathfinding**: A* algorithm with terrain validation
- **🚫 Movement Validation**: Prevents walking through walls and obstacles  
- **📏 Height Detection**: Proper Z-coordinate calculation for terrain
- **⚡ Performance Optimization**: Regional loading based on online players

## ⚠️ **Important Notes**

- **File Format**: Only L2J `.l2j` geodata format is supported
- **Chronicle Compatibility**: These files are specifically for High Five chronicle
- **Memory Management**: Regions are loaded/unloaded dynamically to optimize memory usage

---

[← Back to README](../README.md)