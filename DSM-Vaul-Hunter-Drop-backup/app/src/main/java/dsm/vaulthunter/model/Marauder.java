package dsm.vaulthunter.model;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/**
 * Class representing a Marauder (enemy) in the Vault Hunter game
 */
public class Marauder implements Serializable {
    private int id;
    private String name;
    private int difficulty; // 1-5 scale, higher means more difficult
    private int health;
    private int power;
    private int imageResourceId;
    private MarauderType type;
    
    // Loot that can be obtained when defeating this marauder
    private List<LootItem> possibleLoot;
    
    /**
     * Marauder types
     */
    public enum MarauderType {
        BANDIT("Bandit"),
        ROBOT("Robot"),
        MERCENARY("Mercenary"),
        ASSASSIN("Assassin"),
        MUTANT("Mutant");
        
        private final String displayName;
        
        MarauderType(String displayName) {
            this.displayName = displayName;
        }
        
        public String getDisplayName() {
            return displayName;
        }
    }
    
    /**
     * Constructor
     */
    public Marauder(int id, String name, int difficulty, MarauderType type) {
        this.id = id;
        this.name = name;
        this.difficulty = difficulty;
        this.type = type;
        
        // Calculate stats based on difficulty
        this.health = 50 + (difficulty * 30);
        this.power = 5 + (difficulty * 5);
        
        // Initialize loot
        this.possibleLoot = new ArrayList<>();
        
        // Set image resource - these would be replaced with actual images later
        // this.imageResourceId = getResourceForType(type);
        this.imageResourceId = 0; // Default for now
    }
    
    /**
     * Get possible loot for this marauder
     * @return List of possible loot items
     */
    public List<LootItem> getPossibleLoot() {
        return possibleLoot;
    }
    
    /**
     * Add a possible loot item
     * @param loot Loot item to add
     */
    public void addPossibleLoot(LootItem loot) {
        possibleLoot.add(loot);
    }
    
    /**
     * Generate random loot when marauder is defeated
     * @return List of loot items obtained
     */
    public List<LootItem> generateLoot() {
        List<LootItem> obtainedLoot = new ArrayList<>();
        Random random = new Random();
        
        // Determine number of items to drop based on difficulty
        int itemCount = 1;
        if (random.nextDouble() < difficulty * 0.1) {
            itemCount++;
        }
        
        // Randomly select loot items
        for (int i = 0; i < itemCount && !possibleLoot.isEmpty(); i++) {
            int index = random.nextInt(possibleLoot.size());
            obtainedLoot.add(possibleLoot.get(index));
        }
        
        return obtainedLoot;
    }
    
    // Getters and setters
    
    public int getId() {
        return id;
    }
    
    public String getName() {
        return name;
    }
    
    public int getDifficulty() {
        return difficulty;
    }
    
    public int getHealth() {
        return health;
    }
    
    public void setHealth(int health) {
        this.health = health;
    }
    
    public int getPower() {
        return power;
    }
    
    public int getImageResourceId() {
        return imageResourceId;
    }
    
    public MarauderType getType() {
        return type;
    }
    
    public String getTypeDisplayName() {
        return type.getDisplayName();
    }
    
    /**
     * Check if the marauder is defeated
     * @return true if health is zero or less
     */
    public boolean isDefeated() {
        return health <= 0;
    }
    
    /**
     * Take damage from the player
     * @param damage Amount of damage to take
     * @return true if the marauder is defeated by this damage
     */
    public boolean takeDamage(int damage) {
        health -= damage;
        return isDefeated();
    }
    
    /**
     * Calculate the damage this marauder will do on its turn
     * @return Damage amount
     */
    public int calculateDamage() {
        // Base damage plus random variation
        return power + new Random().nextInt(5);
    }
    
    /**
     * Loot item class - represents items that can be obtained from marauders
     */
    public static class LootItem implements Serializable {
        private int id;
        private String name;
        private String description;
        private LootType type;
        private int value;
        private int quantity;
        private int imageResourceId;
        
        public enum LootType {
            CRYSTAL, // Crystals for upgrading treasures
            CURRENCY, // In-game currency
            ITEM // General items like health packs, shields, etc.
        }
        
        /**
         * Constructor
         */
        public LootItem(int id, String name, String description, LootType type, int value) {
            this.id = id;
            this.name = name;
            this.description = description;
            this.type = type;
            this.value = value;
            this.quantity = 1;
        }
        
        // Getters and setters
        
        public int getId() {
            return id;
        }
        
        public String getName() {
            return name;
        }
        
        public String getDescription() {
            return description;
        }
        
        public LootType getType() {
            return type;
        }
        
        public int getValue() {
            return value;
        }
        
        public int getQuantity() {
            return quantity;
        }
        
        public void setQuantity(int quantity) {
            this.quantity = quantity;
        }
        
        public int getImageResourceId() {
            return imageResourceId;
        }
        
        @Override
        public String toString() {
            return name + (quantity > 1 ? " x" + quantity : "");
        }
    }
}