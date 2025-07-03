package dsm.vaulthunter.model;

import java.io.Serializable;
import java.util.Date;

/**
 * Class representing a treasure that has been collected by a user
 */
public class CollectedTreasure implements Serializable {
    private long id;
    private Treasure treasure;
    private double latitude;
    private double longitude;
    private long collectionTime;
    private int powerLevel;
    private int durability;
    
    /**
     * Constructor for a newly collected treasure
     */
    public CollectedTreasure(Treasure treasure, double latitude, double longitude, long collectionTime) {
        this.treasure = treasure;
        this.latitude = latitude;
        this.longitude = longitude;
        this.collectionTime = collectionTime;
        
        // Generate random stats within range based on treasure's base stats
        this.powerLevel = calculateRandomStat(treasure.getPowerLevel());
        this.durability = calculateRandomStat(treasure.getDurability());
    }
    
    /**
     * Constructor for loading from database
     */
    public CollectedTreasure(long id, Treasure treasure, double latitude, double longitude, 
                             long collectionTime, int powerLevel, int durability) {
        this.id = id;
        this.treasure = treasure;
        this.latitude = latitude;
        this.longitude = longitude;
        this.collectionTime = collectionTime;
        this.powerLevel = powerLevel;
        this.durability = durability;
    }
    
    /**
     * Calculate a random stat value based on the base value
     * @param baseStat The base stat value
     * @return A random value between 80% and 120% of base
     */
    private int calculateRandomStat(int baseStat) {
        double multiplier = 0.8 + (Math.random() * 0.4); // Random between 0.8 and 1.2
        return (int) (baseStat * multiplier);
    }
    
    public long getId() {
        return id;
    }
    
    public void setId(long id) {
        this.id = id;
    }
    
    public Treasure getTreasure() {
        return treasure;
    }
    
    public void setTreasure(Treasure treasure) {
        this.treasure = treasure;
    }
    
    public double getLatitude() {
        return latitude;
    }
    
    public void setLatitude(double latitude) {
        this.latitude = latitude;
    }
    
    public double getLongitude() {
        return longitude;
    }
    
    public void setLongitude(double longitude) {
        this.longitude = longitude;
    }
    
    public long getCollectionTime() {
        return collectionTime;
    }
    
    public void setCollectionTime(long collectionTime) {
        this.collectionTime = collectionTime;
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
    
    /**
     * Get a formatted date string of when this treasure was collected
     * @return Formatted date string
     */
    public String getCollectionDateString() {
        return new Date(collectionTime).toString();
    }
    
    @Override
    public String toString() {
        return treasure.getName() + " (PL: " + powerLevel + ", DUR: " + durability + ")";
    }
}