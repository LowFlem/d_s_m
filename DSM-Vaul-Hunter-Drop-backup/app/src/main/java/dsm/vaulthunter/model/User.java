package dsm.vaulthunter.model;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Class representing a user in the Vault Hunter game
 */
public class User implements Serializable {
    private long id;
    private String name;
    private String email;
    private String password;
    private char gender;
    private CharacterType characterType;
    private int level;
    private int experiencePoints;
    private List<CollectedTreasure> collectedTreasures;
    private Map<Treasure, Integer> treasureCount;
    private Map<Integer, Crystal> crystals;
    private String lastLatitude;
    private String lastLongitude;
    private long tokensEarned;
    private long tokensWithdrawn;
    
    /**
     * Constructor for creating a new user
     */
    public User(String name, String email, String password, char gender) {
        this.name = name;
        this.email = email;
        this.password = password;
        this.gender = gender;
        this.level = 1;
        this.experiencePoints = 0;
        this.collectedTreasures = new ArrayList<>();
        this.treasureCount = new HashMap<>();
        this.crystals = new HashMap<>();
        this.tokensEarned = 0;
        this.tokensWithdrawn = 0;
        
        // Set default character type based on gender
        if (gender == 'F') {
            this.characterType = CharacterType.FEMALE_HUMAN_1;
        } else {
            this.characterType = CharacterType.MALE_HUMAN;
        }
    }
    
    /**
     * Constructor with character type
     */
    public User(String name, String email, String password, CharacterType characterType) {
        this.name = name;
        this.email = email;
        this.password = password;
        this.characterType = characterType;
        this.gender = characterType.getGender();
        this.level = 1;
        this.experiencePoints = 0;
        this.collectedTreasures = new ArrayList<>();
        this.treasureCount = new HashMap<>();
        this.crystals = new HashMap<>();
        this.tokensEarned = 0;
        this.tokensWithdrawn = 0;
    }
    
    /**
     * Constructor for loading a user from database
     */
    public User(long id, String name, String email, String password, char gender, 
                CharacterType characterType, int level, int experiencePoints, 
                String lastLatitude, String lastLongitude, 
                long tokensEarned, long tokensWithdrawn) {
        this.id = id;
        this.name = name;
        this.email = email;
        this.password = password;
        this.gender = gender;
        this.characterType = characterType;
        this.level = level;
        this.experiencePoints = experiencePoints;
        this.lastLatitude = lastLatitude;
        this.lastLongitude = lastLongitude;
        this.collectedTreasures = new ArrayList<>();
        this.treasureCount = new HashMap<>();
        this.crystals = new HashMap<>();
        this.tokensEarned = tokensEarned;
        this.tokensWithdrawn = tokensWithdrawn;
    }
    
    /**
     * Add a collected treasure to the user's collection
     * @param appearance The treasure appearance that was collected
     */
    public void collectTreasure(TreasureAppearance appearance) {
        Treasure treasure = appearance.getTreasure();
        
        // Create a new collected treasure
        CollectedTreasure collectedTreasure = new CollectedTreasure(
                appearance.getTreasure(),
                appearance.getLatitude(),
                appearance.getLongitude(),
                System.currentTimeMillis()
        );
        
        // Add to collection
        collectedTreasures.add(collectedTreasure);
        
        // Update count
        Integer count = treasureCount.get(treasure);
        if (count == null) {
            treasureCount.put(treasure, 1);
        } else {
            treasureCount.put(treasure, count + 1);
        }
        
        // Add crystal if found
        if (treasure.getCrystal() != null) {
            Crystal crystal = treasure.getCrystal();
            Crystal userCrystal = crystals.get(crystal.getId());
            
            if (userCrystal == null) {
                // First time getting this crystal type
                userCrystal = new Crystal(crystal.getId(), crystal.getName(), crystal.getType(), crystal.getIconResource());
                crystals.put(crystal.getId(), userCrystal);
            }
            
            // Add random amount of crystals (1-3)
            int amount = (int) (Math.random() * 3) + 1;
            userCrystal.addQuantity(amount);
        }
    }
    
    /**
     * Claim tokens from a vault
     * @param vault The vault to claim tokens from
     * @return true if claim was successful
     */
    public boolean claimVaultTokens(Vault vault) {
        if (vault.getStatus() != Vault.VaultStatus.CLAIMED) {
            return false;
        }
        
        // Add tokens to user's total
        tokensEarned += vault.getTokenAmount();
        
        // Mark vault as withdrawn
        return vault.withdraw();
    }
    
    /**
     * Get the count of a specific treasure type that the user has collected
     * @param treasure The treasure to check
     * @return The number of this treasure collected
     */
    public int getTreasureCount(Treasure treasure) {
        Integer count = treasureCount.get(treasure);
        return count != null ? count : 0;
    }
    
    /**
     * Add experience points to the user
     * @param amount Amount of XP to add
     * @return true if user leveled up
     */
    public boolean addExperiencePoints(int amount) {
        int oldLevel = level;
        experiencePoints += amount;
        
        // Check for level up
        calculateLevel();
        
        return level > oldLevel;
    }
    
    /**
     * Calculate user level based on experience points
     */
    private void calculateLevel() {
        // Level formula: level = 1 + sqrt(xp / 100)
        level = 1 + (int) Math.sqrt(experiencePoints / 100);
    }
    
    /**
     * Attempt to upgrade a treasure using crystals
     * @param treasure The treasure to upgrade
     * @return true if upgrade was successful
     */
    public boolean upgradeTreasure(CollectedTreasure treasure) {
        if (!treasure.getTreasure().canUpgrade()) {
            return false;
        }
        
        Treasure originalTreasure = treasure.getTreasure();
        Crystal requiredCrystal = originalTreasure.getCrystal();
        int requiredAmount = originalTreasure.getRequiredCrystals();
        
        // Check if user has the crystal
        Crystal userCrystal = crystals.get(requiredCrystal.getId());
        if (userCrystal == null || userCrystal.getQuantity() < requiredAmount) {
            return false;
        }
        
        // Use crystals
        userCrystal.useQuantity(requiredAmount);
        
        // Upgrade treasure
        treasure.setTreasure(originalTreasure.getUpgrade());
        
        return true;
    }
    
    // Getters and setters
    
    public long getId() {
        return id;
    }
    
    public void setId(long id) {
        this.id = id;
    }
    
    public String getName() {
        return name;
    }
    
    public void setName(String name) {
        this.name = name;
    }
    
    public String getEmail() {
        return email;
    }
    
    public void setEmail(String email) {
        this.email = email;
    }
    
    public String getPassword() {
        return password;
    }
    
    public void setPassword(String password) {
        this.password = password;
    }
    
    public char getGender() {
        return gender;
    }
    
    public void setGender(char gender) {
        this.gender = gender;
    }
    
    public CharacterType getCharacterType() {
        return characterType;
    }
    
    public void setCharacterType(CharacterType characterType) {
        this.characterType = characterType;
        this.gender = characterType.getGender();
    }
    
    public int getLevel() {
        return level;
    }
    
    public void setLevel(int level) {
        this.level = level;
    }
    
    public int getExperiencePoints() {
        return experiencePoints;
    }
    
    public void setExperiencePoints(int experiencePoints) {
        this.experiencePoints = experiencePoints;
        calculateLevel();
    }
    
    public List<CollectedTreasure> getCollectedTreasures() {
        return collectedTreasures;
    }
    
    public void setCollectedTreasures(List<CollectedTreasure> collectedTreasures) {
        this.collectedTreasures = collectedTreasures;
        
        // Rebuild the count map
        treasureCount.clear();
        for (CollectedTreasure collected : collectedTreasures) {
            Treasure treasure = collected.getTreasure();
            Integer count = treasureCount.get(treasure);
            if (count == null) {
                treasureCount.put(treasure, 1);
            } else {
                treasureCount.put(treasure, count + 1);
            }
        }
    }
    
    public Map<Integer, Crystal> getCrystals() {
        return crystals;
    }
    
    public void setCrystals(Map<Integer, Crystal> crystals) {
        this.crystals = crystals;
    }
    
    public String getLastLatitude() {
        return lastLatitude;
    }
    
    public void setLastLatitude(String lastLatitude) {
        this.lastLatitude = lastLatitude;
    }
    
    public String getLastLongitude() {
        return lastLongitude;
    }
    
    public void setLastLongitude(String lastLongitude) {
        this.lastLongitude = lastLongitude;
    }
    
    public long getTokensEarned() {
        return tokensEarned;
    }
    
    public void setTokensEarned(long tokensEarned) {
        this.tokensEarned = tokensEarned;
    }
    
    public long getTokensWithdrawn() {
        return tokensWithdrawn;
    }
    
    public void setTokensWithdrawn(long tokensWithdrawn) {
        this.tokensWithdrawn = tokensWithdrawn;
    }
    
    public long getAvailableTokens() {
        return tokensEarned - tokensWithdrawn;
    }
    
    @Override
    public String toString() {
        return name + " (Level " + level + ")";
    }
}