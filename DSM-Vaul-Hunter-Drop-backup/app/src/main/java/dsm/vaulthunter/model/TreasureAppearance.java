package dsm.vaulthunter.model;

import java.io.Serializable;

/**
 * Class representing a treasure appearing in the world
 */
public class TreasureAppearance implements Serializable {
    private Treasure treasure;
    private double latitude;
    private double longitude;
    private long appearanceTime;
    private long expirationTime;
    private String eventId; // Associated event ID, if any
    
    /**
     * Constructor for a new treasure appearance
     */
    public TreasureAppearance(Treasure treasure, double latitude, double longitude) {
        this.treasure = treasure;
        this.latitude = latitude;
        this.longitude = longitude;
        this.appearanceTime = System.currentTimeMillis();
        
        // Default expiration is 30 minutes after appearance
        this.expirationTime = appearanceTime + (30 * 60 * 1000);
    }
    
    /**
     * Constructor with custom expiration time
     */
    public TreasureAppearance(Treasure treasure, double latitude, double longitude, long expirationTime) {
        this.treasure = treasure;
        this.latitude = latitude;
        this.longitude = longitude;
        this.appearanceTime = System.currentTimeMillis();
        this.expirationTime = expirationTime;
    }
    
    /**
     * Constructor with event association
     */
    public TreasureAppearance(Treasure treasure, double latitude, double longitude, long expirationTime, String eventId) {
        this.treasure = treasure;
        this.latitude = latitude;
        this.longitude = longitude;
        this.appearanceTime = System.currentTimeMillis();
        this.expirationTime = expirationTime;
        this.eventId = eventId;
    }
    
    /**
     * Check if this appearance has expired
     * @return true if expired
     */
    public boolean isExpired() {
        return System.currentTimeMillis() > expirationTime;
    }
    
    /**
     * Calculate distance from the user's location
     * @param userLatitude User's latitude
     * @param userLongitude User's longitude
     * @return Distance in meters
     */
    public float distanceFromUser(double userLatitude, double userLongitude) {
        // Rough distance calculation using the Haversine formula
        final int R = 6371000; // Earth radius in meters
        
        double latDistance = Math.toRadians(userLatitude - latitude);
        double lonDistance = Math.toRadians(userLongitude - longitude);
        
        double a = Math.sin(latDistance / 2) * Math.sin(latDistance / 2)
                + Math.cos(Math.toRadians(latitude)) * Math.cos(Math.toRadians(userLatitude))
                * Math.sin(lonDistance / 2) * Math.sin(lonDistance / 2);
                
        double c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
        
        return (float) (R * c);
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
    
    public long getAppearanceTime() {
        return appearanceTime;
    }
    
    public void setAppearanceTime(long appearanceTime) {
        this.appearanceTime = appearanceTime;
    }
    
    public long getExpirationTime() {
        return expirationTime;
    }
    
    public void setExpirationTime(long expirationTime) {
        this.expirationTime = expirationTime;
    }
    
    public String getEventId() {
        return eventId;
    }
    
    public void setEventId(String eventId) {
        this.eventId = eventId;
    }
    
    @Override
    public String toString() {
        return treasure.getName() + " at [" + latitude + ", " + longitude + "]";
    }
}