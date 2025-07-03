package dsm.vaulthunter.util;

import android.content.ContentValues;
import android.content.Context;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.database.sqlite.SQLiteOpenHelper;
import android.util.Log;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.CollectedTreasure;
import dsm.vaulthunter.model.Crystal;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.TreasureType;
import dsm.vaulthunter.model.User;

/**
 * SQLite database helper for the Vault Hunter game
 */
public class DatabaseHelper extends SQLiteOpenHelper {
    private static final String TAG = "DatabaseHelper";
    
    // Singleton instance
    private static DatabaseHelper instance;
    
    // Database metadata
    private static final String DATABASE_NAME = "vaulthunter.db";
    private static final int DATABASE_VERSION = 1;
    
    // Table names
    private static final String TABLE_USERS = "users";
    private static final String TABLE_TREASURE_TYPES = "treasure_types";
    private static final String TABLE_TREASURES = "treasures";
    private static final String TABLE_CRYSTALS = "crystals";
    private static final String TABLE_COLLECTED_TREASURES = "collected_treasures";
    private static final String TABLE_USER_CRYSTALS = "user_crystals";
    private static final String TABLE_VAULTS = "vaults";
    
    // Common column names
    private static final String KEY_ID = "id";
    private static final String KEY_NAME = "name";
    
    // USER table columns
    private static final String KEY_EMAIL = "email";
    private static final String KEY_PASSWORD = "password";
    private static final String KEY_GENDER = "gender";
    private static final String KEY_LEVEL = "level";
    private static final String KEY_XP = "experience_points";
    private static final String KEY_LAST_LAT = "last_latitude";
    private static final String KEY_LAST_LONG = "last_longitude";
    
    // TREASURE_TYPES table columns
    private static final String KEY_DESCRIPTION = "description";
    private static final String KEY_ICON = "icon_resource";
    
    // TREASURES table columns
    private static final String KEY_NUMBER = "number";
    private static final String KEY_POWER = "power_level";
    private static final String KEY_DURABILITY = "durability";
    private static final String KEY_WEIGHT = "weight";
    private static final String KEY_SIZE = "size";
    private static final String KEY_RARITY = "rarity";
    private static final String KEY_PRIMARY_TYPE = "primary_type_id";
    private static final String KEY_SECONDARY_TYPE = "secondary_type_id";
    private static final String KEY_CRYSTAL = "crystal_id";
    private static final String KEY_DISTANCE = "distance";
    private static final String KEY_CRYSTALS_TO_UPGRADE = "crystals_to_upgrade";
    private static final String KEY_UPGRADE = "upgrade_id";
    
    // CRYSTALS table columns
    private static final String KEY_TYPE_ID = "type_id";
    
    // COLLECTED_TREASURES table columns
    private static final String KEY_TREASURE_ID = "treasure_id";
    private static final String KEY_USER_ID = "user_id";
    private static final String KEY_LATITUDE = "latitude";
    private static final String KEY_LONGITUDE = "longitude";
    private static final String KEY_COLLECTION_TIME = "collection_time";
    
    // USER_CRYSTALS table columns
    private static final String KEY_CRYSTAL_ID = "crystal_id";
    private static final String KEY_QUANTITY = "quantity";
    
    // VAULTS table columns
    private static final String KEY_VAULT_ID = "vault_id";
    private static final String KEY_UNLOCK_TIME = "unlock_timestamp";
    private static final String KEY_EXPIRY_TIME = "expiry_timestamp";
    private static final String KEY_CLAIMED_TIME = "claimed_timestamp";
    private static final String KEY_WITHDRAWN_TIME = "withdrawn_timestamp";
    private static final String KEY_DIFFICULTY = "difficulty_level";
    private static final String KEY_TOKEN_AMOUNT = "token_amount";
    private static final String KEY_CLAIMED_BY = "claimed_by_user_id";
    private static final String KEY_STATUS = "status";
    private static final String KEY_REQUIRED_DISTANCE = "required_distance";
    
    /**
     * Private constructor for singleton pattern
     */
    private DatabaseHelper(Context context) {
        super(context, DATABASE_NAME, null, DATABASE_VERSION);
    }
    
    /**
     * Get the singleton instance
     * @param context Application context
     * @return The DatabaseHelper instance
     */
    public static synchronized DatabaseHelper getInstance(Context context) {
        if (instance == null) {
            instance = new DatabaseHelper(context.getApplicationContext());
        }
        return instance;
    }
    
    @Override
    public void onCreate(SQLiteDatabase db) {
        // Create USERS table
        String CREATE_USERS_TABLE = "CREATE TABLE " + TABLE_USERS + "("
                + KEY_ID + " INTEGER PRIMARY KEY AUTOINCREMENT,"
                + KEY_NAME + " TEXT,"
                + KEY_EMAIL + " TEXT UNIQUE,"
                + KEY_PASSWORD + " TEXT,"
                + KEY_GENDER + " TEXT,"
                + KEY_LEVEL + " INTEGER,"
                + KEY_XP + " INTEGER,"
                + KEY_LAST_LAT + " TEXT,"
                + KEY_LAST_LONG + " TEXT,"
                + "tokens_earned INTEGER DEFAULT 0,"
                + "tokens_withdrawn INTEGER DEFAULT 0" + ")";
        
        // Create TREASURE_TYPES table
        String CREATE_TREASURE_TYPES_TABLE = "CREATE TABLE " + TABLE_TREASURE_TYPES + "("
                + KEY_ID + " INTEGER PRIMARY KEY,"
                + KEY_NAME + " TEXT,"
                + KEY_DESCRIPTION + " TEXT,"
                + KEY_ICON + " INTEGER" + ")";
        
        // Create TREASURES table
        String CREATE_TREASURES_TABLE = "CREATE TABLE " + TABLE_TREASURES + "("
                + KEY_ID + " INTEGER PRIMARY KEY AUTOINCREMENT,"
                + KEY_NUMBER + " INTEGER UNIQUE,"
                + KEY_NAME + " TEXT,"
                + KEY_DESCRIPTION + " TEXT,"
                + KEY_POWER + " INTEGER,"
                + KEY_DURABILITY + " INTEGER,"
                + KEY_WEIGHT + " REAL,"
                + KEY_SIZE + " REAL,"
                + KEY_RARITY + " INTEGER,"
                + KEY_PRIMARY_TYPE + " INTEGER,"
                + KEY_SECONDARY_TYPE + " INTEGER,"
                + KEY_CRYSTAL + " INTEGER,"
                + KEY_DISTANCE + " REAL,"
                + KEY_CRYSTALS_TO_UPGRADE + " INTEGER,"
                + KEY_UPGRADE + " INTEGER,"
                + "FOREIGN KEY(" + KEY_PRIMARY_TYPE + ") REFERENCES " + TABLE_TREASURE_TYPES + "(" + KEY_ID + "),"
                + "FOREIGN KEY(" + KEY_SECONDARY_TYPE + ") REFERENCES " + TABLE_TREASURE_TYPES + "(" + KEY_ID + "),"
                + "FOREIGN KEY(" + KEY_CRYSTAL + ") REFERENCES " + TABLE_CRYSTALS + "(" + KEY_ID + "),"
                + "FOREIGN KEY(" + KEY_UPGRADE + ") REFERENCES " + TABLE_TREASURES + "(" + KEY_ID + ")" + ")";
        
        // Create CRYSTALS table
        String CREATE_CRYSTALS_TABLE = "CREATE TABLE " + TABLE_CRYSTALS + "("
                + KEY_ID + " INTEGER PRIMARY KEY,"
                + KEY_NAME + " TEXT,"
                + KEY_TYPE_ID + " INTEGER,"
                + KEY_ICON + " INTEGER,"
                + "FOREIGN KEY(" + KEY_TYPE_ID + ") REFERENCES " + TABLE_TREASURE_TYPES + "(" + KEY_ID + ")" + ")";
        
        // Create COLLECTED_TREASURES table
        String CREATE_COLLECTED_TREASURES_TABLE = "CREATE TABLE " + TABLE_COLLECTED_TREASURES + "("
                + KEY_ID + " INTEGER PRIMARY KEY AUTOINCREMENT,"
                + KEY_TREASURE_ID + " INTEGER,"
                + KEY_USER_ID + " INTEGER,"
                + KEY_LATITUDE + " REAL,"
                + KEY_LONGITUDE + " REAL,"
                + KEY_COLLECTION_TIME + " INTEGER,"
                + KEY_POWER + " INTEGER,"
                + KEY_DURABILITY + " INTEGER,"
                + "FOREIGN KEY(" + KEY_TREASURE_ID + ") REFERENCES " + TABLE_TREASURES + "(" + KEY_ID + "),"
                + "FOREIGN KEY(" + KEY_USER_ID + ") REFERENCES " + TABLE_USERS + "(" + KEY_ID + ")" + ")";
        
        // Create USER_CRYSTALS table
        String CREATE_USER_CRYSTALS_TABLE = "CREATE TABLE " + TABLE_USER_CRYSTALS + "("
                + KEY_ID + " INTEGER PRIMARY KEY AUTOINCREMENT,"
                + KEY_USER_ID + " INTEGER,"
                + KEY_CRYSTAL_ID + " INTEGER,"
                + KEY_QUANTITY + " INTEGER,"
                + "FOREIGN KEY(" + KEY_USER_ID + ") REFERENCES " + TABLE_USERS + "(" + KEY_ID + "),"
                + "FOREIGN KEY(" + KEY_CRYSTAL_ID + ") REFERENCES " + TABLE_CRYSTALS + "(" + KEY_ID + ")" + ")";
        
        // Create VAULTS table
        String CREATE_VAULTS_TABLE = "CREATE TABLE " + TABLE_VAULTS + "("
                + KEY_ID + " INTEGER PRIMARY KEY AUTOINCREMENT,"
                + KEY_VAULT_ID + " TEXT UNIQUE,"
                + KEY_NAME + " TEXT,"
                + KEY_LATITUDE + " REAL,"
                + KEY_LONGITUDE + " REAL,"
                + KEY_UNLOCK_TIME + " INTEGER,"
                + KEY_EXPIRY_TIME + " INTEGER,"
                + KEY_CLAIMED_TIME + " INTEGER,"
                + KEY_WITHDRAWN_TIME + " INTEGER,"
                + KEY_DIFFICULTY + " INTEGER,"
                + KEY_TOKEN_AMOUNT + " INTEGER,"
                + KEY_CLAIMED_BY + " INTEGER,"
                + KEY_STATUS + " INTEGER,"
                + KEY_REQUIRED_DISTANCE + " REAL,"
                + "FOREIGN KEY(" + KEY_CLAIMED_BY + ") REFERENCES " + TABLE_USERS + "(" + KEY_ID + ")" + ")";
        
        // Execute create table statements
        db.execSQL(CREATE_USERS_TABLE);
        db.execSQL(CREATE_TREASURE_TYPES_TABLE);
        db.execSQL(CREATE_CRYSTALS_TABLE);
        db.execSQL(CREATE_TREASURES_TABLE);
        db.execSQL(CREATE_COLLECTED_TREASURES_TABLE);
        db.execSQL(CREATE_USER_CRYSTALS_TABLE);
        db.execSQL(CREATE_VAULTS_TABLE);
        
        // Initialize basic data
        initializeTreasureTypes(db);
        initializeCrystals(db);
        initializeTreasures(db);
    }
    
    @Override
    public void onUpgrade(SQLiteDatabase db, int oldVersion, int newVersion) {
        // Drop tables and recreate
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_VAULTS);
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_USER_CRYSTALS);
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_COLLECTED_TREASURES);
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_TREASURES);
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_CRYSTALS);
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_TREASURE_TYPES);
        db.execSQL("DROP TABLE IF EXISTS " + TABLE_USERS);
        
        // Recreate tables
        onCreate(db);
    }
    
    /**
     * Check if a user with the specified email exists
     * @param email Email to check
     * @return true if user exists
     */
    public boolean userExists(String email) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABLE_USERS + " WHERE " + KEY_EMAIL + " = ?";
        Cursor cursor = db.rawQuery(query, new String[] { email });
        
        boolean exists = cursor.getCount() > 0;
        
        cursor.close();
        return exists;
    }
    
    /**
     * Add a new user to the database
     * @param user User to add
     * @return User ID if successful, -1 if failed
     */
    public long addUser(User user) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues values = new ContentValues();
        values.put(KEY_NAME, user.getName());
        values.put(KEY_EMAIL, user.getEmail());
        values.put(KEY_PASSWORD, user.getPassword());
        values.put(KEY_GENDER, String.valueOf(user.getGender()));
        values.put(KEY_LEVEL, user.getLevel());
        values.put(KEY_XP, user.getExperiencePoints());
        values.put(KEY_LAST_LAT, user.getLastLatitude());
        values.put(KEY_LAST_LONG, user.getLastLongitude());
        
        long id = db.insert(TABLE_USERS, null, values);
        
        if (id > 0) {
            user.setId(id);
        }
        
        return id;
    }
    
    /**
     * Authenticate a user
     * @param email User email
     * @param password User password
     * @return User object if authenticated, null otherwise
     */
    public User authenticateUser(String email, String password) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABLE_USERS + 
                      " WHERE " + KEY_EMAIL + " = ? AND " + KEY_PASSWORD + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[] { email, password });
        
        User user = null;
        
        if (cursor.moveToFirst()) {
            user = new User(
                cursor.getLong(cursor.getColumnIndex(KEY_ID)),
                cursor.getString(cursor.getColumnIndex(KEY_NAME)),
                cursor.getString(cursor.getColumnIndex(KEY_EMAIL)),
                cursor.getString(cursor.getColumnIndex(KEY_PASSWORD)),
                cursor.getString(cursor.getColumnIndex(KEY_GENDER)).charAt(0),
                cursor.getInt(cursor.getColumnIndex(KEY_LEVEL)),
                cursor.getInt(cursor.getColumnIndex(KEY_XP)),
                cursor.getString(cursor.getColumnIndex(KEY_LAST_LAT)),
                cursor.getString(cursor.getColumnIndex(KEY_LAST_LONG))
            );
            
            // Load collected treasures
            loadUserCollectedTreasures(user);
            
            // Load user crystals
            loadUserCrystals(user);
        }
        
        cursor.close();
        return user;
    }
    
    /**
     * Update a user's information
     * @param user User to update
     * @return Number of rows affected
     */
    public int updateUser(User user) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues values = new ContentValues();
        values.put(KEY_NAME, user.getName());
        values.put(KEY_LEVEL, user.getLevel());
        values.put(KEY_XP, user.getExperiencePoints());
        values.put(KEY_LAST_LAT, user.getLastLatitude());
        values.put(KEY_LAST_LONG, user.getLastLongitude());
        
        return db.update(TABLE_USERS, values, KEY_ID + " = ?", new String[] { String.valueOf(user.getId()) });
    }
    
    /**
     * Load a user's collected treasures
     * @param user User to load treasures for
     */
    private void loadUserCollectedTreasures(User user) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABLE_COLLECTED_TREASURES + 
                      " WHERE " + KEY_USER_ID + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[] { String.valueOf(user.getId()) });
        
        List<CollectedTreasure> collectedTreasures = new ArrayList<>();
        
        if (cursor.moveToFirst()) {
            do {
                long treasureId = cursor.getLong(cursor.getColumnIndex(KEY_TREASURE_ID));
                Treasure treasure = getTreasureById(treasureId);
                
                if (treasure != null) {
                    CollectedTreasure collected = new CollectedTreasure(
                        cursor.getLong(cursor.getColumnIndex(KEY_ID)),
                        treasure,
                        cursor.getDouble(cursor.getColumnIndex(KEY_LATITUDE)),
                        cursor.getDouble(cursor.getColumnIndex(KEY_LONGITUDE)),
                        cursor.getLong(cursor.getColumnIndex(KEY_COLLECTION_TIME)),
                        cursor.getInt(cursor.getColumnIndex(KEY_POWER)),
                        cursor.getInt(cursor.getColumnIndex(KEY_DURABILITY))
                    );
                    
                    collectedTreasures.add(collected);
                }
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        
        user.setCollectedTreasures(collectedTreasures);
    }
    
    /**
     * Load a user's crystals
     * @param user User to load crystals for
     */
    private void loadUserCrystals(User user) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT uc." + KEY_CRYSTAL_ID + ", uc." + KEY_QUANTITY + 
                      ", c." + KEY_NAME + ", c." + KEY_TYPE_ID + ", c." + KEY_ICON +
                      " FROM " + TABLE_USER_CRYSTALS + " uc" +
                      " JOIN " + TABLE_CRYSTALS + " c ON uc." + KEY_CRYSTAL_ID + " = c." + KEY_ID +
                      " WHERE uc." + KEY_USER_ID + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[] { String.valueOf(user.getId()) });
        
        Map<Integer, Crystal> crystals = new HashMap<>();
        
        if (cursor.moveToFirst()) {
            do {
                int crystalId = cursor.getInt(cursor.getColumnIndex(KEY_CRYSTAL_ID));
                int typeId = cursor.getInt(cursor.getColumnIndex(KEY_TYPE_ID));
                
                TreasureType type = getTreasureTypeById(typeId);
                
                if (type != null) {
                    Crystal crystal = new Crystal(
                        crystalId,
                        cursor.getString(cursor.getColumnIndex(KEY_NAME)),
                        type,
                        cursor.getInt(cursor.getColumnIndex(KEY_ICON))
                    );
                    
                    crystal.setQuantity(cursor.getInt(cursor.getColumnIndex(KEY_QUANTITY)));
                    crystals.put(crystalId, crystal);
                }
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        
        user.setCrystals(crystals);
    }
    
    /**
     * Get a treasure by ID
     * @param id Treasure ID
     * @return Treasure object or null if not found
     */
    private Treasure getTreasureById(long id) {
        // In a real implementation, this would query the database
        // For simplicity, we'll use the AppController's treasure list
        for (Treasure treasure : AppController.getInstance().getAllTreasures()) {
            if (treasure.getNumber() == id) {
                return treasure;
            }
        }
        return null;
    }
    
    /**
     * Get a treasure type by ID
     * @param id Type ID
     * @return TreasureType object or null if not found
     */
    private TreasureType getTreasureTypeById(int id) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABLE_TREASURE_TYPES + " WHERE " + KEY_ID + " = ?";
        Cursor cursor = db.rawQuery(query, new String[] { String.valueOf(id) });
        
        TreasureType type = null;
        
        if (cursor.moveToFirst()) {
            type = new TreasureType(
                cursor.getInt(cursor.getColumnIndex(KEY_ID)),
                cursor.getString(cursor.getColumnIndex(KEY_NAME)),
                cursor.getString(cursor.getColumnIndex(KEY_DESCRIPTION)),
                cursor.getInt(cursor.getColumnIndex(KEY_ICON))
            );
        }
        
        cursor.close();
        return type;
    }
    
    /**
     * Add a collected treasure to the database
     * @param userId User ID
     * @param collectedTreasure Collected treasure
     * @return Collected treasure ID if successful, -1 if failed
     */
    public long addCollectedTreasure(long userId, CollectedTreasure collectedTreasure) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues values = new ContentValues();
        values.put(KEY_TREASURE_ID, collectedTreasure.getTreasure().getNumber());
        values.put(KEY_USER_ID, userId);
        values.put(KEY_LATITUDE, collectedTreasure.getLatitude());
        values.put(KEY_LONGITUDE, collectedTreasure.getLongitude());
        values.put(KEY_COLLECTION_TIME, collectedTreasure.getCollectionTime());
        values.put(KEY_POWER, collectedTreasure.getPowerLevel());
        values.put(KEY_DURABILITY, collectedTreasure.getDurability());
        
        long id = db.insert(TABLE_COLLECTED_TREASURES, null, values);
        
        if (id > 0) {
            collectedTreasure.setId(id);
        }
        
        return id;
    }
    
    /**
     * Update user crystal quantity
     * @param userId User ID
     * @param crystalId Crystal ID
     * @param quantity New quantity
     * @return true if successful
     */
    public boolean updateUserCrystal(long userId, int crystalId, int quantity) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        // Check if this user already has this crystal
        String query = "SELECT * FROM " + TABLE_USER_CRYSTALS + 
                      " WHERE " + KEY_USER_ID + " = ? AND " + KEY_CRYSTAL_ID + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[] { 
            String.valueOf(userId), 
            String.valueOf(crystalId) 
        });
        
        boolean result;
        
        if (cursor.getCount() > 0) {
            // Update existing record
            ContentValues values = new ContentValues();
            values.put(KEY_QUANTITY, quantity);
            
            int rowsAffected = db.update(TABLE_USER_CRYSTALS, values, 
                                        KEY_USER_ID + " = ? AND " + KEY_CRYSTAL_ID + " = ?", 
                                        new String[] { 
                                            String.valueOf(userId), 
                                            String.valueOf(crystalId) 
                                        });
            
            result = rowsAffected > 0;
        } else {
            // Insert new record
            ContentValues values = new ContentValues();
            values.put(KEY_USER_ID, userId);
            values.put(KEY_CRYSTAL_ID, crystalId);
            values.put(KEY_QUANTITY, quantity);
            
            long id = db.insert(TABLE_USER_CRYSTALS, null, values);
            result = id > 0;
        }
        
        cursor.close();
        return result;
    }
    
    /**
     * Initialize treasure types in the database
     * @param db Database
     */
    private void initializeTreasureTypes(SQLiteDatabase db) {
        // Add basic treasure types
        ContentValues values = new ContentValues();
        
        values.put(KEY_ID, 1);
        values.put(KEY_NAME, "Ancient");
        values.put(KEY_DESCRIPTION, "Relics from ancient civilizations");
        values.put(KEY_ICON, 0); // Icon resource ID would be set here
        db.insert(TABLE_TREASURE_TYPES, null, values);
        
        values.clear();
        values.put(KEY_ID, 2);
        values.put(KEY_NAME, "Precious");
        values.put(KEY_DESCRIPTION, "Valuable metals and gemstones");
        values.put(KEY_ICON, 0);
        db.insert(TABLE_TREASURE_TYPES, null, values);
        
        values.clear();
        values.put(KEY_ID, 3);
        values.put(KEY_NAME, "Mystical");
        values.put(KEY_DESCRIPTION, "Items with magical properties");
        values.put(KEY_ICON, 0);
        db.insert(TABLE_TREASURE_TYPES, null, values);
        
        values.clear();
        values.put(KEY_ID, 4);
        values.put(KEY_NAME, "Technological");
        values.put(KEY_DESCRIPTION, "Advanced technological artifacts");
        values.put(KEY_ICON, 0);
        db.insert(TABLE_TREASURE_TYPES, null, values);
        
        values.clear();
        values.put(KEY_ID, 5);
        values.put(KEY_NAME, "Cursed");
        values.put(KEY_DESCRIPTION, "Items with dark powers");
        values.put(KEY_ICON, 0);
        db.insert(TABLE_TREASURE_TYPES, null, values);
    }
    
    /**
     * Initialize crystals in the database
     * @param db Database
     */
    private void initializeCrystals(SQLiteDatabase db) {
        // Add crystals for each treasure type
        ContentValues values = new ContentValues();
        
        for (int i = 1; i <= 5; i++) {
            String name = "";
            switch (i) {
                case 1: name = "Ancient Crystal"; break;
                case 2: name = "Precious Crystal"; break;
                case 3: name = "Mystical Crystal"; break;
                case 4: name = "Technological Crystal"; break;
                case 5: name = "Cursed Crystal"; break;
            }
            
            values.clear();
            values.put(KEY_ID, i);
            values.put(KEY_NAME, name);
            values.put(KEY_TYPE_ID, i);
            values.put(KEY_ICON, 0); // Icon resource ID would be set here
            db.insert(TABLE_CRYSTALS, null, values);
        }
    }
    
    /**
     * Initialize treasures in the database
     * @param db Database
     */
    private void initializeTreasures(SQLiteDatabase db) {
        // In a full implementation, this would add all treasures to the database
        // For simplicity in this skeleton, we'll initialize treasures through the AppController
    }
    
    /**
     * Add a vault to the database
     * @param vault Vault to add
     * @return Vault ID if successful, -1 if failed
     */
    public long addVault(Vault vault) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues values = new ContentValues();
        values.put(KEY_VAULT_ID, vault.getVaultId());
        values.put(KEY_NAME, vault.getName());
        values.put(KEY_LATITUDE, vault.getLatitude());
        values.put(KEY_LONGITUDE, vault.getLongitude());
        values.put(KEY_UNLOCK_TIME, vault.getUnlockTimestamp());
        values.put(KEY_EXPIRY_TIME, vault.getExpiryTimestamp());
        values.put(KEY_CLAIMED_TIME, vault.getClaimedTimestamp());
        values.put(KEY_WITHDRAWN_TIME, vault.getWithdrawnTimestamp());
        values.put(KEY_DIFFICULTY, vault.getDifficultyLevel());
        values.put(KEY_TOKEN_AMOUNT, vault.getTokenAmount());
        
        if (vault.getClaimedBy() != null) {
            values.put(KEY_CLAIMED_BY, vault.getClaimedBy().getId());
        }
        
        values.put(KEY_STATUS, getStatusCode(vault.getStatus()));
        values.put(KEY_REQUIRED_DISTANCE, vault.getRequiredDistance());
        
        long id = db.insert(TABLE_VAULTS, null, values);
        
        if (id > 0) {
            vault.setId(id);
        }
        
        return id;
    }
    
    /**
     * Update a vault in the database
     * @param vault Vault to update
     * @return Number of rows affected
     */
    public int updateVault(Vault vault) {
        SQLiteDatabase db = this.getWritableDatabase();
        
        ContentValues values = new ContentValues();
        values.put(KEY_CLAIMED_TIME, vault.getClaimedTimestamp());
        values.put(KEY_WITHDRAWN_TIME, vault.getWithdrawnTimestamp());
        
        if (vault.getClaimedBy() != null) {
            values.put(KEY_CLAIMED_BY, vault.getClaimedBy().getId());
        }
        
        values.put(KEY_STATUS, getStatusCode(vault.getStatus()));
        
        return db.update(TABLE_VAULTS, values, KEY_ID + " = ?", new String[] { String.valueOf(vault.getId()) });
    }
    
    /**
     * Get a list of vaults for a specific user
     * @param user User to get vaults for
     * @return List of vaults
     */
    public List<Vault> getVaultsForUser(User user) {
        List<Vault> vaults = new ArrayList<>();
        SQLiteDatabase db = this.getReadableDatabase();
        
        // Get vaults claimed by this user
        String query = "SELECT * FROM " + TABLE_VAULTS + 
                      " WHERE " + KEY_CLAIMED_BY + " = ?";
        
        Cursor cursor = db.rawQuery(query, new String[] { String.valueOf(user.getId()) });
        
        if (cursor.moveToFirst()) {
            do {
                Vault vault = createVaultFromCursor(cursor, user);
                vaults.add(vault);
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        
        return vaults;
    }
    
    /**
     * Get vaults near a location
     * @param latitude Latitude
     * @param longitude Longitude
     * @param maxDistance Maximum distance in meters
     * @return List of nearby vaults
     */
    public List<Vault> getNearbyVaults(double latitude, double longitude, double maxDistance) {
        List<Vault> vaults = new ArrayList<>();
        SQLiteDatabase db = this.getReadableDatabase();
        
        // Get all vaults
        String query = "SELECT * FROM " + TABLE_VAULTS;
        Cursor cursor = db.rawQuery(query, null);
        
        if (cursor.moveToFirst()) {
            do {
                Vault vault = createVaultFromCursor(cursor, null);
                
                // Calculate distance and filter
                double distance = vault.calculateDistance(latitude, longitude);
                if (distance <= maxDistance) {
                    vaults.add(vault);
                }
                
            } while (cursor.moveToNext());
        }
        
        cursor.close();
        
        return vaults;
    }
    
    /**
     * Get a vault by ID
     * @param id Vault ID
     * @return Vault object or null if not found
     */
    public Vault getVaultById(long id) {
        SQLiteDatabase db = this.getReadableDatabase();
        
        String query = "SELECT * FROM " + TABLE_VAULTS + " WHERE " + KEY_ID + " = ?";
        Cursor cursor = db.rawQuery(query, new String[] { String.valueOf(id) });
        
        Vault vault = null;
        
        if (cursor.moveToFirst()) {
            vault = createVaultFromCursor(cursor, null);
        }
        
        cursor.close();
        return vault;
    }
    
    /**
     * Create a Vault object from a database cursor
     * @param cursor Database cursor positioned at the vault record
     * @param claimedByUser User who claimed the vault (can be null)
     * @return Vault object
     */
    private Vault createVaultFromCursor(Cursor cursor, User claimedByUser) {
        long id = cursor.getLong(cursor.getColumnIndex(KEY_ID));
        String vaultId = cursor.getString(cursor.getColumnIndex(KEY_VAULT_ID));
        String name = cursor.getString(cursor.getColumnIndex(KEY_NAME));
        double latitude = cursor.getDouble(cursor.getColumnIndex(KEY_LATITUDE));
        double longitude = cursor.getDouble(cursor.getColumnIndex(KEY_LONGITUDE));
        long unlockTimestamp = cursor.getLong(cursor.getColumnIndex(KEY_UNLOCK_TIME));
        long expiryTimestamp = cursor.getLong(cursor.getColumnIndex(KEY_EXPIRY_TIME));
        long claimedTimestamp = cursor.getLong(cursor.getColumnIndex(KEY_CLAIMED_TIME));
        long withdrawnTimestamp = cursor.getLong(cursor.getColumnIndex(KEY_WITHDRAWN_TIME));
        int difficultyLevel = cursor.getInt(cursor.getColumnIndex(KEY_DIFFICULTY));
        long tokenAmount = cursor.getLong(cursor.getColumnIndex(KEY_TOKEN_AMOUNT));
        int statusCode = cursor.getInt(cursor.getColumnIndex(KEY_STATUS));
        double requiredDistance = cursor.getDouble(cursor.getColumnIndex(KEY_REQUIRED_DISTANCE));
        
        // Get claimed by user if not provided
        User user = claimedByUser;
        if (user == null && !cursor.isNull(cursor.getColumnIndex(KEY_CLAIMED_BY))) {
            long userId = cursor.getLong(cursor.getColumnIndex(KEY_CLAIMED_BY));
            // In a real implementation, this would query the user
            // For simplicity, we'll use a placeholder
            user = new User("Unknown", "unknown@example.com", "password", 'M');
            user.setId(userId);
        }
        
        return new Vault(
            id, vaultId, name, latitude, longitude,
            unlockTimestamp, expiryTimestamp, claimedTimestamp, withdrawnTimestamp,
            difficultyLevel, tokenAmount,
            getStatusFromCode(statusCode), user, requiredDistance
        );
    }
    
    /**
     * Convert VaultStatus enum to integer code for database storage
     * @param status VaultStatus enum value
     * @return Integer code
     */
    private int getStatusCode(Vault.VaultStatus status) {
        switch (status) {
            case LOCKED: return 0;
            case UNLOCKED: return 1;
            case CLAIMED: return 2;
            case WITHDRAWN: return 3;
            case EXPIRED: return 4;
            default: return 0;
        }
    }
    
    /**
     * Convert integer code to VaultStatus enum
     * @param code Integer code
     * @return VaultStatus enum value
     */
    private Vault.VaultStatus getStatusFromCode(int code) {
        switch (code) {
            case 0: return Vault.VaultStatus.LOCKED;
            case 1: return Vault.VaultStatus.UNLOCKED;
            case 2: return Vault.VaultStatus.CLAIMED;
            case 3: return Vault.VaultStatus.WITHDRAWN;
            case 4: return Vault.VaultStatus.EXPIRED;
            default: return Vault.VaultStatus.LOCKED;
        }
    }
}