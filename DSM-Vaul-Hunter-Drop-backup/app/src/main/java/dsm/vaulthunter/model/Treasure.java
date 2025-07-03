package dsm.vaulthunter.model;

/**
 * Class representing a Treasure in the Vault Hunter game
 */
public class Treasure {
    private int number;
    private String name;
    private String description;
    private int powerLevel;
    private int durability;
    private double weight;
    private double size;
    private int rarity; // 1 (common) to 5 (legendary)
    private TreasureType primaryType;
    private TreasureType secondaryType;
    private Crystal crystal;
    private double distance; // Distance to unlock Vault (in km)
    private Integer crystalsToUpgrade; // Number of crystals needed to upgrade
    private Treasure upgrade; // Next upgrade level

    public Treasure(int number, String name, String description, int powerLevel, int durability, 
                  double weight, double size, int rarity, TreasureType primaryType, 
                  TreasureType secondaryType, Crystal crystal, double distance) {
        this.number = number;
        this.name = name;
        this.description = description;
        this.powerLevel = powerLevel;
        this.durability = durability;
        this.weight = weight;
        this.size = size;
        this.rarity = rarity;
        this.primaryType = primaryType;
        this.secondaryType = secondaryType;
        this.crystal = crystal;
        this.distance = distance;
    }

    public int getNumber() {
        return number;
    }

    public void setNumber(int number) {
        this.number = number;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public int getPowerLevel() {
        return powerLevel;
    }

    public void setPowerLevel(int powerLevel) {
        this.powerLevel = powerLevel;
    }

    public int getDurability() {
        return durability;
    }

    public void setDurability(int durability) {
        this.durability = durability;
    }

    public double getWeight() {
        return weight;
    }

    public void setWeight(double weight) {
        this.weight = weight;
    }

    public double getSize() {
        return size;
    }

    public void setSize(double size) {
        this.size = size;
    }

    public int getRarity() {
        return rarity;
    }

    public void setRarity(int rarity) {
        this.rarity = rarity;
    }

    public TreasureType getPrimaryType() {
        return primaryType;
    }

    public void setPrimaryType(TreasureType primaryType) {
        this.primaryType = primaryType;
    }

    public TreasureType getSecondaryType() {
        return secondaryType;
    }

    public void setSecondaryType(TreasureType secondaryType) {
        this.secondaryType = secondaryType;
    }

    public Crystal getCrystal() {
        return crystal;
    }

    public void setCrystal(Crystal crystal) {
        this.crystal = crystal;
    }

    public double getDistance() {
        return distance;
    }

    public void setDistance(double distance) {
        this.distance = distance;
    }

    public Integer getCrystalsToUpgrade() {
        return crystalsToUpgrade;
    }

    public void setCrystalsToUpgrade(Integer crystalsToUpgrade) {
        this.crystalsToUpgrade = crystalsToUpgrade;
    }

    public Treasure getUpgrade() {
        return upgrade;
    }

    public void setUpgrade(Treasure upgrade) {
        this.upgrade = upgrade;
    }
    
    /**
     * Checks if the Treasure can be upgraded
     * @return true if the Treasure can be upgraded
     */
    public boolean canUpgrade() {
        return upgrade != null && crystalsToUpgrade != null;
    }
    
    /**
     * Returns the name of the Treasure's upgrade stage
     * @return Name of the upgrade stage
     */
    public String getUpgradeStage() {
        // Check if it's the first stage
        if (number % 3 == 1) {
            return "Basic stage";
        } 
        // Check if it's the last stage
        else if (!canUpgrade()) {
            return "Final stage";
        } 
        // Otherwise, it's an intermediate stage
        else {
            return "Intermediate stage";
        }
    }
    
    /**
     * Returns the string with the Treasure's types
     * @return String with the type(s)
     */
    public String getTypesString() {
        if (secondaryType != null) {
            return primaryType.getName() + " / " + secondaryType.getName();
        } else {
            return primaryType.getName();
        }
    }

    @Override
    public String toString() {
        return "#" + number + " " + name;
    }

    public int getRequiredCrystals() {
        return crystalsToUpgrade != null ? crystalsToUpgrade : 0;
    }
}