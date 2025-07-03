package dsm.vaulthunter.model;

/**
 * Enum representing character types in the Vault Hunter game
 */
public enum CharacterType {
    // Original character types
    MALE_HUMAN("Male Human", 'M', "A skilled treasure hunter with expertise in unlocking vaults", 1.0f, 1.2f, 1.0f, 1.0f),
    FEMALE_HUMAN("Female Human", 'F', "A resourceful bounty hunter with a talent for finding valuable treasures", 1.0f, 1.0f, 1.0f, 1.2f),
    
    // New character types based on the artwork
    ROBOT("Android", 'O', "An advanced android with enhanced scanning abilities", 1.3f, 1.0f, 1.0f, 1.0f),
    FEMALE_SKULL("Calavera", 'F', "A mysterious hunter with the ability to sense valuable crystals", 1.0f, 1.0f, 1.3f, 1.0f);
    
    private final String name;
    private final char gender; // 'M' for male, 'F' for female, 'O' for other
    private final String description;
    
    // Multipliers for different abilities
    private final float scanningMultiplier; // Affects treasure detection range
    private final float unlockingMultiplier; // Affects vault unlocking speed
    private final float crystalMultiplier; // Affects crystal drop rate
    private final float tokenMultiplier; // Affects token rewards
    
    /**
     * Constructor
     * @param name Character type name
     * @param gender Character gender code
     * @param description Character description
     * @param scanningMultiplier Multiplier for scanning abilities
     * @param unlockingMultiplier Multiplier for vault unlocking abilities
     * @param crystalMultiplier Multiplier for crystal finding abilities
     * @param tokenMultiplier Multiplier for token rewards
     */
    CharacterType(String name, char gender, String description, 
                 float scanningMultiplier, float unlockingMultiplier, 
                 float crystalMultiplier, float tokenMultiplier) {
        this.name = name;
        this.gender = gender;
        this.description = description;
        this.scanningMultiplier = scanningMultiplier;
        this.unlockingMultiplier = unlockingMultiplier;
        this.crystalMultiplier = crystalMultiplier;
        this.tokenMultiplier = tokenMultiplier;
    }
    
    /**
     * Get the character type name
     * @return Character type name
     */
    public String getName() {
        return name;
    }
    
    /**
     * Get the gender code
     * @return Gender code ('M', 'F', or 'O')
     */
    public char getGender() {
        return gender;
    }
    
    /**
     * Get the character description
     * @return Character description
     */
    public String getDescription() {
        return description;
    }
    
    /**
     * Get the scanning multiplier
     * @return Scanning multiplier
     */
    public float getScanningMultiplier() {
        return scanningMultiplier;
    }
    
    /**
     * Get the unlocking multiplier
     * @return Unlocking multiplier
     */
    public float getUnlockingMultiplier() {
        return unlockingMultiplier;
    }
    
    /**
     * Get the crystal multiplier
     * @return Crystal multiplier
     */
    public float getCrystalMultiplier() {
        return crystalMultiplier;
    }
    
    /**
     * Get the token multiplier
     * @return Token multiplier
     */
    public float getTokenMultiplier() {
        return tokenMultiplier;
    }
    
    /**
     * Get the resource name for the character image
     * @return Resource name for character image
     */
    public String getImageResourceName() {
        return "character_" + name().toLowerCase();
    }
    
    /**
     * Get a detailed description including abilities
     * @return Detailed description
     */
    public String getDetailedDescription() {
        StringBuilder description = new StringBuilder(this.description);
        description.append("\n\nSpecial Abilities:");
        
        if (scanningMultiplier > 1.0f) {
            description.append("\n• Enhanced scanning (").append(Math.round((scanningMultiplier - 1.0f) * 100)).append("% bonus)");
        }
        
        if (unlockingMultiplier > 1.0f) {
            description.append("\n• Faster vault unlocking (").append(Math.round((unlockingMultiplier - 1.0f) * 100)).append("% bonus)");
        }
        
        if (crystalMultiplier > 1.0f) {
            description.append("\n• Increased crystal finding (").append(Math.round((crystalMultiplier - 1.0f) * 100)).append("% bonus)");
        }
        
        if (tokenMultiplier > 1.0f) {
            description.append("\n• Extra token rewards (").append(Math.round((tokenMultiplier - 1.0f) * 100)).append("% bonus)");
        }
        
        return description.toString();
    }
}