package teste.lucasvegi.pokemongooffline.Model;

/**
 * Class representing a Treasure in the game
 */
public class Treasure {
    private int number;
    private String name;
    private String description;
    private int cpBase;
    private int hpBase;
    private double weight;
    private double height;
    private int rarity; // 1 (common) to 5 (legendary)
    private Tipo primaryType;
    private Tipo secondaryType;
    private Doce crystal;
    private double distance; // Distance to unlock Vault (in km)
    private Integer crystalsToUpgrade; // Number of crystals needed to upgrade
    private Treasure upgrade; // Next upgrade level

    public Treasure(int number, String name, String description, int cpBase, int hpBase, 
                  double weight, double height, int rarity, Tipo primaryType, 
                  Tipo secondaryType, Doce crystal, double distance) {
        this.number = number;
        this.name = name;
        this.description = description;
        this.cpBase = cpBase;
        this.hpBase = hpBase;
        this.weight = weight;
        this.height = height;
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

    public int getCpBase() {
        return cpBase;
    }

    public void setCpBase(int cpBase) {
        this.cpBase = cpBase;
    }

    public int getHpBase() {
        return hpBase;
    }

    public void setHpBase(int hpBase) {
        this.hpBase = hpBase;
    }

    public double getWeight() {
        return weight;
    }

    public void setWeight(double weight) {
        this.weight = weight;
    }

    public double getHeight() {
        return height;
    }

    public void setHeight(double height) {
        this.height = height;
    }

    public int getRarity() {
        return rarity;
    }

    public void setRarity(int rarity) {
        this.rarity = rarity;
    }

    public Tipo getPrimaryType() {
        return primaryType;
    }

    public void setPrimaryType(Tipo primaryType) {
        this.primaryType = primaryType;
    }

    public Tipo getSecondaryType() {
        return secondaryType;
    }

    public void setSecondaryType(Tipo secondaryType) {
        this.secondaryType = secondaryType;
    }

    public Doce getCrystal() {
        return crystal;
    }

    public void setCrystal(Doce crystal) {
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
        return 0;
    }
}