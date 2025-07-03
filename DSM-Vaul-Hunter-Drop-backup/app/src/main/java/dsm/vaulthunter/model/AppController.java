package dsm.vaulthunter.model;

import android.content.Context;
import android.util.Log;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;

import dsm.vaulthunter.util.DatabaseHelper;
import dsm.vaulthunter.util.RandomUtil;

/**
 * Main controller for the application
 * Handles users, treasures, vaults, and database operations
 */
public class AppController {
    private static final String TAG = "AppController";
    
    // Singleton instance
    private static AppController instance;
    
    private Context appContext;
    private DatabaseHelper dbHelper;
    private User loggedInUser;
    private List<Treasure> allTreasures;
    private List<TreasureType> treasureTypes;
    private List<Crystal> crystalTypes;
    private Map<String, Vault> vaultCache;
    
    /**
     * Private constructor for singleton pattern
     */
    private AppController() {
        allTreasures = new ArrayList<>();
        treasureTypes = new ArrayList<>();
        crystalTypes = new ArrayList<>();
        vaultCache = new HashMap<>();
    }
    
    /**
     * Get the singleton instance
     * @return The AppController instance
     */
    public static synchronized AppController getInstance() {
        if (instance == null) {
            instance = new AppController();
        }
        return instance;
    }
    
    /**
     * Initialize the controller with application context
     * @param context Application context
     */
    public void initialize(Context context) {
        this.appContext = context.getApplicationContext();
        this.dbHelper = DatabaseHelper.getInstance(appContext);
        
        // Initialize data
        initializeTreasureTypes();
        initializeCrystals();
        initializeTreasures();
    }
    
    /**
     * Initialize treasure types
     */
    private void initializeTreasureTypes() {
        treasureTypes = new ArrayList<>();
        
        // Add basic treasure types
        treasureTypes.add(new TreasureType(1, "Ancient", "Relics from ancient civilizations", 0));
        treasureTypes.add(new TreasureType(2, "Precious", "Valuable metals and gemstones", 0));
        treasureTypes.add(new TreasureType(3, "Mystical", "Items with magical properties", 0));
        treasureTypes.add(new TreasureType(4, "Technological", "Advanced technological artifacts", 0));
        treasureTypes.add(new TreasureType(5, "Cursed", "Items with dark powers", 0));
    }
    
    /**
     * Initialize crystals
     */
    private void initializeCrystals() {
        crystalTypes = new ArrayList<>();
        
        // Add crystals for each treasure type
        for (TreasureType type : treasureTypes) {
            String name = type.getName();
            crystalTypes.add(new Crystal(type.getId(), name, type, 0));
        }
    }
    
    /**
     * Initialize treasures
     */
    private void initializeTreasures() {
        allTreasures = new ArrayList<>();
        
        // Add sample treasures
        Treasure t1 = new Treasure(1, "Golden Amulet", "An ancient golden amulet with mysterious engravings",
                500, 100, 0.5, 5.0, 3, treasureTypes.get(1), treasureTypes.get(0), crystalTypes.get(1), 0.5);
        
        Treasure t2 = new Treasure(2, "Crystal Shard", "A glowing crystal shard that pulses with energy",
                300, 50, 0.2, 3.0, 2, treasureTypes.get(2), null, crystalTypes.get(2), 0.3);
        
        Treasure t3 = new Treasure(3, "Alien Artifact", "A strange technological device of unknown origin",
                800, 120, 1.0, 10.0, 4, treasureTypes.get(3), treasureTypes.get(2), crystalTypes.get(3), 1.0);
        
        // Set up upgrade path
        Treasure t1Upgraded = new Treasure(4, "Enhanced Golden Amulet", "An ancient golden amulet, enhanced with mystical power",
                800, 150, 0.5, 5.0, 4, treasureTypes.get(1), treasureTypes.get(2), crystalTypes.get(1), 0.5);
        
        t1.setCrystalsToUpgrade(5);
        t1.setUpgrade(t1Upgraded);
        
        // Add to list
        allTreasures.add(t1);
        allTreasures.add(t2);
        allTreasures.add(t3);
        allTreasures.add(t1Upgraded);
    }
    
    /**
     * Register a new user
     * @param name User name
     * @param email User email
     * @param password User password
     * @param gender User gender
     * @return true if registration was successful
     */
    public boolean registerUser(String name, String email, String password, char gender) {
        if (dbHelper.userExists(email)) {
            return false;
        }
        
        User user = new User(name, email, password, gender);
        long id = dbHelper.addUser(user);
        
        return id > 0;
    }
    
    /**
     * Login a user
     * @param email User email
     * @param password User password
     * @return true if login was successful
     */
    public boolean loginUser(String email, String password) {
        User user = dbHelper.authenticateUser(email, password);
        
        if (user != null) {
            this.loggedInUser = user;
            return true;
        }
        
        return false;
    }
    
    /**
     * Logout the current user
     */
    public void logoutUser() {
        this.loggedInUser = null;
    }
    
    /**
     * Get the currently logged in user
     * @return The logged in user or null if none
     */
    public User getLoggedInUser() {
        return loggedInUser;
    }
    
    /**
     * Update user information in the database
     * @param user User to update
     * @return true if update was successful
     */
    public boolean updateUser(User user) {
        int rowsAffected = dbHelper.updateUser(user);
        
        if (user.getId() == loggedInUser.getId()) {
            loggedInUser = user;
        }
        
        return rowsAffected > 0;
    }
    
    /**
     * Get all treasures
     * @return List of all treasures
     */
    public List<Treasure> getAllTreasures() {
        return allTreasures;
    }
    
    /**
     * Get a treasure by number
     * @param number Treasure number
     * @return Treasure object or null if not found
     */
    public Treasure getTreasureByNumber(int number) {
        for (Treasure treasure : allTreasures) {
            if (treasure.getNumber() == number) {
                return treasure;
            }
        }
        return null;
    }
    
    /**
     * Generate a random treasure appearance at a location
     * @param latitude Location latitude
     * @param longitude Location longitude
     * @return TreasureAppearance object
     */
    public TreasureAppearance generateRandomTreasure(double latitude, double longitude) {
        // Get a random treasure
        int index = RandomUtil.getRandomInt(0, allTreasures.size() - 1);
        Treasure treasure = allTreasures.get(index);
        
        // Create a small random offset (up to 50 meters)
        double offsetLat = (Math.random() - 0.5) * 0.001; // Approx 50m in latitude
        double offsetLon = (Math.random() - 0.5) * 0.001; // Approx 50m in longitude
        
        // Create the appearance
        return new TreasureAppearance(
                treasure,
                latitude + offsetLat,
                longitude + offsetLon,
                System.currentTimeMillis(),
                System.currentTimeMillis() + 1000 * 60 * 15 // 15 minutes expiry
        );
    }
    
    /**
     * Add a collected treasure for the current user
     * @param appearance The treasure appearance that was collected
     * @return true if collection was successful
     */
    public boolean collectTreasure(TreasureAppearance appearance) {
        if (loggedInUser == null) {
            return false;
        }
        
        // Add to user's collection
        loggedInUser.collectTreasure(appearance);
        
        // Create a CollectedTreasure object
        CollectedTreasure collected = loggedInUser.getCollectedTreasures()
                .get(loggedInUser.getCollectedTreasures().size() - 1);
        
        // Save to database
        long id = dbHelper.addCollectedTreasure(loggedInUser.getId(), collected);
        
        if (id > 0) {
            collected.setId(id);
            
            // Update crystal count if changed
            if (appearance.getTreasure().getCrystal() != null) {
                Crystal crystal = appearance.getTreasure().getCrystal();
                Crystal userCrystal = loggedInUser.getCrystals().get(crystal.getId());
                
                if (userCrystal != null) {
                    dbHelper.updateUserCrystal(
                            loggedInUser.getId(),
                            userCrystal.getId(),
                            userCrystal.getQuantity()
                    );
                }
            }
            
            return true;
        }
        
        return false;
    }
    
    /**
     * Get all vaults for the current user
     * @return List of vaults
     */
    public List<Vault> getUserVaults(User user) {
        if (user == null) {
            return new ArrayList<>();
        }
        
        return dbHelper.getVaultsForUser(user);
    }
    
    /**
     * Get nearby vaults
     * @param latitude Current latitude
     * @param longitude Current longitude
     * @param maxDistance Maximum distance in meters
     * @return List of nearby vaults
     */
    public List<Vault> getNearbyVaults(double latitude, double longitude, double maxDistance) {
        return dbHelper.getNearbyVaults(latitude, longitude, maxDistance);
    }
    
    /**
     * Get a vault by ID
     * @param id Vault ID
     * @return Vault object or null if not found
     */
    public Vault getVaultById(long id) {
        // Check cache first
        String cacheKey = "id:" + id;
        if (vaultCache.containsKey(cacheKey)) {
            return vaultCache.get(cacheKey);
        }
        
        // Get from database
        Vault vault = dbHelper.getVaultById(id);
        
        // Cache for future use
        if (vault != null) {
            vaultCache.put(cacheKey, vault);
        }
        
        return vault;
    }
    
    /**
     * Add a new vault
     * @param vault Vault to add
     * @return true if successful
     */
    public boolean addVault(Vault vault) {
        long id = dbHelper.addVault(vault);
        
        if (id > 0) {
            vault.setId(id);
            vaultCache.put("id:" + id, vault);
            return true;
        }
        
        return false;
    }
    
    /**
     * Update a vault
     * @param vault Vault to update
     * @return true if successful
     */
    public boolean updateVault(Vault vault) {
        int rowsAffected = dbHelper.updateVault(vault);
        
        if (rowsAffected > 0) {
            vaultCache.put("id:" + vault.getId(), vault);
            return true;
        }
        
        return false;
    }
    
    /**
     * Generate a random vault near a location
     * @param latitude Center latitude
     * @param longitude Center longitude
     * @param radius Radius in meters for random placement
     * @return Generated vault
     */
    public Vault generateRandomVault(double latitude, double longitude, double radius) {
        // Create random location within radius
        double angle = Math.random() * 2 * Math.PI;
        double distance = Math.random() * radius;
        
        // Convert to lat/lon offset (approximate)
        double earthRadius = 6371000; // meters
        double latOffset = Math.cos(angle) * distance / earthRadius;
        double lonOffset = Math.sin(angle) * distance / (earthRadius * Math.cos(Math.toRadians(latitude)));
        
        double vaultLat = latitude + Math.toDegrees(latOffset);
        double vaultLon = longitude + Math.toDegrees(lonOffset);
        
        // Generate random properties
        String vaultId = "vault-" + RandomUtil.getRandomString(8);
        String name = getRandomVaultName();
        
        // Timing - a vault that unlocks between now and 2 days from now
        long now = System.currentTimeMillis();
        long unlockTime = now + RandomUtil.getRandomInt(0, 2 * 24 * 60 * 60 * 1000);
        long expiryTime = unlockTime + RandomUtil.getRandomInt(1, 7) * 24 * 60 * 60 * 1000; // 1-7 days after unlock
        
        // Other properties
        int difficulty = RandomUtil.getRandomInt(1, 5);
        long tokenAmount = 100 + difficulty * RandomUtil.getRandomInt(50, 200); // 150-1100 tokens based on difficulty
        double requiredDistance = 25 + (5 - difficulty) * 5; // 30-50 meters, easier vaults have larger radius
        
        // Create the vault
        Vault vault = new Vault(
                vaultId, name, vaultLat, vaultLon,
                unlockTime, expiryTime, difficulty,
                tokenAmount, requiredDistance
        );
        
        // Save to database
        addVault(vault);
        
        return vault;
    }
    
    /**
     * Generate a random vault name
     * @return Random vault name
     */
    private String getRandomVaultName() {
        String[] adjectives = {"Ancient", "Hidden", "Secret", "Lost", "Forgotten", "Mysterious", "Enchanted", 
                              "Cursed", "Gilded", "Royal", "Sacred", "Haunted", "Elemental", "Celestial"};
        
        String[] nouns = {"Vault", "Treasury", "Cache", "Trove", "Chest", "Coffer", "Reliquary", 
                         "Strongbox", "Repository", "Hoard", "Stash", "Collection"};
        
        String adj = adjectives[RandomUtil.getRandomInt(0, adjectives.length - 1)];
        String noun = nouns[RandomUtil.getRandomInt(0, nouns.length - 1)];
        
        return adj + " " + noun;
    }
    
    /**
     * Clear the cache
     */
    public void clearCache() {
        vaultCache.clear();
    }
}