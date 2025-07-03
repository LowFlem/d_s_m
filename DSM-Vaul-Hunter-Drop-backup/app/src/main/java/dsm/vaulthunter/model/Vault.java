package dsm.vaulthunter.model;

import java.io.Serializable;

/**
 * Class representing a Vault in the Vault Hunter game
 */
public class Vault implements Serializable {
    private long id;
    private String vaultId;
    private String name;
    private double latitude;
    private double longitude;
    private long unlockTimestamp;
    private long expiryTimestamp;
    private long claimedTimestamp;
    private long withdrawnTimestamp;
    private int difficultyLevel;
    private long tokenAmount;
    private User claimedBy;
    private VaultStatus status;
    private double requiredDistance;
    
    /**
     * Enum representing possible Vault statuses
     */
    public enum VaultStatus {
        LOCKED,      // Vault has not been unlocked yet
        UNLOCKED,    // Vault is unlocked but not claimed
        CLAIMED,     // Vault has been claimed but tokens not withdrawn
        WITHDRAWN,   // Vault tokens have been withdrawn
        EXPIRED      // Vault expired without being claimed
    }
    
    /**
     * Constructor for a new Vault
     */
    public Vault(String vaultId, String name, double latitude, double longitude, 
                long unlockTimestamp, long expiryTimestamp, int difficultyLevel, 
                long tokenAmount, double requiredDistance) {
        this.vaultId = vaultId;
        this.name = name;
        this.latitude = latitude;
        this.longitude = longitude;
        this.unlockTimestamp = unlockTimestamp;
        this.expiryTimestamp = expiryTimestamp;
        this.difficultyLevel = difficultyLevel;
        this.tokenAmount = tokenAmount;
        this.requiredDistance = requiredDistance;
        this.status = VaultStatus.LOCKED;
    }
    
    /**
     * Constructor with ID for database loading
     */
    public Vault(long id, String vaultId, String name, double latitude, double longitude, 
                long unlockTimestamp, long expiryTimestamp, long claimedTimestamp,
                long withdrawnTimestamp, int difficultyLevel, long tokenAmount,
                VaultStatus status, User claimedBy, double requiredDistance) {
        this.id = id;
        this.vaultId = vaultId;
        this.name = name;
        this.latitude = latitude;
        this.longitude = longitude;
        this.unlockTimestamp = unlockTimestamp;
        this.expiryTimestamp = expiryTimestamp;
        this.claimedTimestamp = claimedTimestamp;
        this.withdrawnTimestamp = withdrawnTimestamp;
        this.difficultyLevel = difficultyLevel;
        this.tokenAmount = tokenAmount;
        this.status = status;
        this.claimedBy = claimedBy;
        this.requiredDistance = requiredDistance;
    }
    
    /**
     * Unlock the vault
     * @return true if the vault was unlocked
     */
    public boolean unlock() {
        if (status == VaultStatus.LOCKED) {
            long currentTime = System.currentTimeMillis();
            if (currentTime >= unlockTimestamp) {
                status = VaultStatus.UNLOCKED;
                return true;
            }
        }
        return false;
    }
    
    /**
     * Check if the vault has expired
     * @return true if the vault has expired
     */
    public boolean checkExpired() {
        if (status == VaultStatus.LOCKED || status == VaultStatus.UNLOCKED) {
            long currentTime = System.currentTimeMillis();
            if (currentTime > expiryTimestamp) {
                status = VaultStatus.EXPIRED;
                return true;
            }
        }
        return false;
    }
    
    /**
     * Claim the vault
     * @param user User claiming the vault
     * @return true if the vault was claimed
     */
    public boolean claim(User user) {
        if (status == VaultStatus.UNLOCKED) {
            status = VaultStatus.CLAIMED;
            claimedBy = user;
            claimedTimestamp = System.currentTimeMillis();
            return true;
        }
        return false;
    }
    
    /**
     * Withdraw tokens from the vault
     * @return true if tokens were withdrawn
     */
    public boolean withdraw() {
        if (status == VaultStatus.CLAIMED) {
            status = VaultStatus.WITHDRAWN;
            withdrawnTimestamp = System.currentTimeMillis();
            return true;
        }
        return false;
    }
    
    /**
     * Calculate distance to user
     * @param userLatitude User latitude
     * @param userLongitude User longitude
     * @return Distance in meters
     */
    public double calculateDistance(double userLatitude, double userLongitude) {
        // Earth's radius in meters
        final double R = 6371000;
        
        double latDistance = Math.toRadians(userLatitude - latitude);
        double lonDistance = Math.toRadians(userLongitude - longitude);
        
        double a = Math.sin(latDistance / 2) * Math.sin(latDistance / 2)
                + Math.cos(Math.toRadians(latitude)) * Math.cos(Math.toRadians(userLatitude))
                * Math.sin(lonDistance / 2) * Math.sin(lonDistance / 2);
        
        double c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
        
        return R * c;
    }
    
    /**
     * Check if user is close enough to interact with the vault
     * @param userLatitude User latitude
     * @param userLongitude User longitude
     * @return true if user is close enough
     */
    public boolean isUserWithinRange(double userLatitude, double userLongitude) {
        return calculateDistance(userLatitude, userLongitude) <= requiredDistance;
    }
    
    // Getters and setters
    
    public long getId() {
        return id;
    }
    
    public void setId(long id) {
        this.id = id;
    }
    
    public String getVaultId() {
        return vaultId;
    }
    
    public void setVaultId(String vaultId) {
        this.vaultId = vaultId;
    }
    
    public String getName() {
        return name;
    }
    
    public void setName(String name) {
        this.name = name;
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
    
    public long getUnlockTimestamp() {
        return unlockTimestamp;
    }
    
    public void setUnlockTimestamp(long unlockTimestamp) {
        this.unlockTimestamp = unlockTimestamp;
    }
    
    public long getExpiryTimestamp() {
        return expiryTimestamp;
    }
    
    public void setExpiryTimestamp(long expiryTimestamp) {
        this.expiryTimestamp = expiryTimestamp;
    }
    
    public long getClaimedTimestamp() {
        return claimedTimestamp;
    }
    
    public void setClaimedTimestamp(long claimedTimestamp) {
        this.claimedTimestamp = claimedTimestamp;
    }
    
    public long getWithdrawnTimestamp() {
        return withdrawnTimestamp;
    }
    
    public void setWithdrawnTimestamp(long withdrawnTimestamp) {
        this.withdrawnTimestamp = withdrawnTimestamp;
    }
    
    public int getDifficultyLevel() {
        return difficultyLevel;
    }
    
    public void setDifficultyLevel(int difficultyLevel) {
        this.difficultyLevel = difficultyLevel;
    }
    
    public long getTokenAmount() {
        return tokenAmount;
    }
    
    public void setTokenAmount(long tokenAmount) {
        this.tokenAmount = tokenAmount;
    }
    
    public User getClaimedBy() {
        return claimedBy;
    }
    
    public void setClaimedBy(User claimedBy) {
        this.claimedBy = claimedBy;
    }
    
    public VaultStatus getStatus() {
        return status;
    }
    
    public void setStatus(VaultStatus status) {
        this.status = status;
    }
    
    public double getRequiredDistance() {
        return requiredDistance;
    }
    
    public void setRequiredDistance(double requiredDistance) {
        this.requiredDistance = requiredDistance;
    }
    
    @Override
    public String toString() {
        return name + " (" + status + ")";
    }
}