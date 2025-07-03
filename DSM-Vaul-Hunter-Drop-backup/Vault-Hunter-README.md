# Vault Hunter AR

## Introduction

Vault Hunter AR is an augmented reality treasure hunting game built on the Decentralized State Machine (DSM) network. The game enables players to explore the real world to discover and collect valuable treasures and unlock special vaults containing DSM tokens.

## Project Structure

This project is built in parallel with the original Pokémon GO clone, gradually transitioning the codebase to the new Vault Hunter game. The project uses an MVC (Model-View-Controller) architecture with the following package structure:

```
dsm.vaulthunter/
├── model/          # Data models and business logic
├── view/           # View adapters and UI components
├── controller/     # Activities and UI controllers
└── util/           # Utility classes and helpers
```

## Features

* **User Registration & Authentication**: Create an account and login securely
* **AR Treasure Hunting**: Use augmented reality with your device's camera and sensors to find and collect treasures
* **Treasure Catalog (TreasureDex)**: Browse and view details of all treasures you've collected
* **Vault System**: Discover special vaults containing DSM tokens around the world
* **DSM Integration**: Connect to the Decentralized State Machine network for token rewards
* **Treasure Upgrading**: Use crystals to upgrade treasures to more powerful forms
* **Real-world Exploration**: Explore the physical world using GPS and maps
* **Trading System**: Trade collected treasures with nearby players

## Technical Implementation

### Core Components

1. **Model Classes**:
   - `Treasure`: Represents collectible items with properties and upgrade paths
   - `Vault`: Represents special token containers with varying rarity levels
   - `User`: Manages user profile, collection, and game progress
   - `AppController`: Singleton facade providing access to game functionality
   - `DSMClient`: Handles communication with the Decentralized State Machine network

2. **Controllers**:
   - `MapActivity`: Main game screen showing the world map with treasures
   - `TreasureCollectionActivity`: AR-based collection screen using camera and sensors
   - `DSMMainActivity`: Gateway to DSM-specific features and events
   - `TreasureHuntActivity`: Special event gameplay for DSM network integration

3. **Utilities**:
   - `DatabaseHelper`: Manages SQLite database operations
   - `MapConfigUtils`: Configures and manages map functionality
   - `PermissionHelper`: Handles runtime permissions
   - `ViewUnitsUtil`: Provides screen size and unit conversion utilities

### AR Implementation

The augmented reality implementation uses:
- Device camera for real-world view
- Gyroscope for treasure movement with device rotation
- Accelerometer for depth perception
- Touch controls for collecting treasures

### DSM Integration

- Connection to DSM bootstrap nodes
- Vault rewards using DSM tokens
- Regional events for special treasure hunts
- Token claiming and withdrawal features

## Development Approach

This project takes a parallel development approach:
1. Keeping the original Pokémon GO clone intact and functional
2. Building a new, clean English codebase with modern architecture
3. Gradually transitioning functionality while maintaining both versions
4. Using shared resources where appropriate but maintaining separation for clarity

## Current Status

The skeletal structure of the new Vault Hunter AR implementation is in place with:
- Core model classes defined
- Basic controller activities created
- Utility classes for common functionality
- DSM integration framework

## Next Steps

1. Design and implement proper layouts for new activities
2. Create database schema and migration paths
3. Replace Pokémon images with treasure and vault graphics
4. Implement Bluetooth trading for treasures
5. Enhance AR collection mechanics
6. Build out the DSM network integration
7. Complete the catalog and profile systems

---

© 2025 DSM Vault Hunter Team