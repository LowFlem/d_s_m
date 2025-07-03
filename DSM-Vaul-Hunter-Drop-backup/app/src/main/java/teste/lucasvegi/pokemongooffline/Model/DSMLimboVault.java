package teste.lucasvegi.pokemongooffline.Model;

import android.location.Location;
import android.util.Log;

import java.io.Serializable;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

/**
 * Class that represents a DSM Limbo Vault treasure chest
 */
public class DSMLimboVault implements Serializable {
    private static final String TAG = "DSMLimboVault";
    
    // Chest variant types
    public enum ChestVariant {
        BRONZE,  // 150,000 tokens ($1,500 value)
        SILVER,  // 700,000 tokens ($7,000 value)
        GOLD     // 1,500,000 tokens ($15,000 value)
    }
    
    // Chest state types
    public enum ChestStatus {
        UNMINTED,
        MINTED, 
        CLAIMED,
        WITHDRAWN
    }
    
    private String chestId;
    private String regionId;
    private ChestVariant variant;
    private ChestStatus status;
    private long tokenAmount;
    private double latitude;
    private double longitude;
    private String claimedBy;
    private long claimedAt;
    private Map<String, String> metadata;
    
    /**
     * Constructor for a new DSM Limbo Vault chest
     */
    public DSMLimboVault(String chestId, String regionId, ChestVariant variant, 
                        double latitude, double longitude) {
        this.chestId = chestId;
        this.regionId = regionId;
        this.variant = variant;
        this.status = ChestStatus.MINTED;
        this.latitude = latitude;
        this.longitude = longitude;
        this.metadata = new HashMap<>();
        
        // Set token amount based on variant
        switch(variant) {
            case BRONZE:
                this.tokenAmount = 150000;
                break;
            case SILVER:
                this.tokenAmount = 700000;
                break;
            case GOLD:
                this.tokenAmount = 1500000;
                break;
        }
    }
    
    /**
     * Checks if the user is within range to claim this chest
     * @param userLatitude User's latitude
     * @param userLongitude User's longitude
     * @param rangeMeters Maximum distance in meters to allow claim
     * @return true if user is within claiming range
     */
    public boolean isWithinClaimRange(double userLatitude, double userLongitude, float rangeMeters) {
        Location chestLocation = new Location("chest");
        chestLocation.setLatitude(this.latitude);
        chestLocation.setLongitude(this.longitude);
        
        Location userLocation = new Location("user");
        userLocation.setLatitude(userLatitude);
        userLocation.setLongitude(userLongitude);
        
        float distance = chestLocation.distanceTo(userLocation);
        
        return distance <= rangeMeters;
    }
    
    /**
     * Claims this chest for the specified user
     * @param userId User ID claiming the chest
     * @param timestamp Timestamp of the claim
     * @return true if claim was successful
     */
    public boolean claim(String userId, long timestamp) {
        if (this.status != ChestStatus.MINTED) {
            Log.e(TAG, "Cannot claim chest " + chestId + " with status " + status);
            return false;
        }
        
        this.status = ChestStatus.CLAIMED;
        this.claimedBy = userId;
        this.claimedAt = timestamp;
        
        // Store claim info in metadata
        this.metadata.put("claimedBy", userId);
        this.metadata.put("claimedAt", String.valueOf(timestamp));
        this.metadata.put("claimRegion", this.regionId);
        
        return true;
    }
    
    /**
     * Transitions this chest to Withdrawn state after tokens are distributed
     * @return true if withdrawal was successful
     */
    public boolean withdraw() {
        if (this.status != ChestStatus.CLAIMED) {
            Log.e(TAG, "Cannot withdraw from chest " + chestId + " with status " + status);
            return false;
        }
        
        this.status = ChestStatus.WITHDRAWN;
        return true;
    }
    
    // Getters and setters
    
    public String getChestId() {
        return chestId;
    }
    
    public String getRegionId() {
        return regionId;
    }
    
    public ChestVariant getVariant() {
        return variant;
    }
    
    public ChestStatus getStatus() {
        return status;
    }
    
    public long getTokenAmount() {
        return tokenAmount;
    }
    
    public double getLatitude() {
        return latitude;
    }
    
    public double getLongitude() {
        return longitude;
    }
    
    public String getClaimedBy() {
        return claimedBy;
    }
    
    public long getClaimedAt() {
        return claimedAt;
    }
    
    public Map<String, String> getMetadata() {
        return metadata;
    }
    
    public void setMetadata(String key, String value) {
        this.metadata.put(key, value);
    }
    
    /**
     * Factory method to create a random chest with specified parameters
     */
    public static DSMLimboVault createRandomChest(String regionId, double latitude, double longitude) {
        String chestId = "CHEST-" + UUID.randomUUID().toString().substring(0, 8).toUpperCase();
        
        // Determine chest variant with weighted randomness
        // Bronze: 80%, Silver: 15%, Gold: 5%
        double random = Math.random();
        ChestVariant variant;
        
        if (random < 0.8) {
            variant = ChestVariant.BRONZE;
        } else if (random < 0.95) {
            variant = ChestVariant.SILVER;
        } else {
            variant = ChestVariant.GOLD;
        }
        
        return new DSMLimboVault(chestId, regionId, variant, latitude, longitude);
    }
    
    /**
     * Get the drawable resource for this chest variant
     * @return Drawable resource name for the chest image
     */
    public String getChestDrawableResource() {
        switch(variant) {
            case BRONZE:
                return "dsm_bronze_chest";
            case SILVER:
                return "dsm_silver_chest";
            case GOLD:
                return "dsm_gold_chest";
            default:
                return "dsm_bronze_chest";
        }
    }
    
    @Override
    public String toString() {
        return "Chest " + chestId + " (" + variant + ")";
    }
}