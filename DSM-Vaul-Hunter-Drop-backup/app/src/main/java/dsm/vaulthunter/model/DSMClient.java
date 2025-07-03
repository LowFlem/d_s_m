package dsm.vaulthunter.model;

import android.util.Log;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;

/**
 * Client for the Decentralized State Machine (DSM) network
 */
public class DSMClient {
    private static final String TAG = "DSMClient";
    
    // Singleton instance
    private static DSMClient instance;
    
    private String networkEndpoint;
    private boolean connected;
    private List<RegionEvent> activeEvents;
    
    /**
     * Private constructor for singleton pattern
     */
    private DSMClient() {
        this.connected = false;
        this.activeEvents = new ArrayList<>();
        initializeTestEvents(); // For development testing
    }
    
    /**
     * Get the singleton instance
     * @return The DSMClient instance
     */
    public static synchronized DSMClient getInstance() {
        if (instance == null) {
            instance = new DSMClient();
        }
        return instance;
    }
    
    /**
     * Connect to the DSM network
     * @param endpoint The bootstrap endpoint to connect to
     * @return true if connection was successful
     */
    public boolean connect(String endpoint) {
        this.networkEndpoint = endpoint;
        Log.d(TAG, "Connecting to DSM endpoint: " + endpoint);
        
        // TODO: Implement actual network connection
        // For now, simulate successful connection
        this.connected = true;
        
        return connected;
    }
    
    /**
     * Check if client is connected to the DSM network
     * @return true if connected
     */
    public boolean isConnected() {
        return connected;
    }
    
    /**
     * Disconnect from the DSM network
     */
    public void disconnect() {
        if (connected) {
            Log.d(TAG, "Disconnecting from DSM endpoint: " + networkEndpoint);
            connected = false;
        }
    }
    
    /**
     * Get the current active event in the user's region
     * @return The active RegionEvent or null if none
     */
    public RegionEvent getCurrentActiveEvent() {
        long currentTime = System.currentTimeMillis();
        
        for (RegionEvent event : activeEvents) {
            if (event.getStartTime() <= currentTime && event.getEndTime() >= currentTime) {
                return event;
            }
        }
        
        return null;
    }
    
    /**
     * Get all upcoming events
     * @return List of upcoming events
     */
    public List<RegionEvent> getUpcomingEvents() {
        List<RegionEvent> upcoming = new ArrayList<>();
        long currentTime = System.currentTimeMillis();
        
        for (RegionEvent event : activeEvents) {
            if (event.getStartTime() > currentTime) {
                upcoming.add(event);
            }
        }
        
        return upcoming;
    }
    
    /**
     * Initialize some test events for development
     */
    private void initializeTestEvents() {
        // Get current time
        long now = System.currentTimeMillis();
        
        // Create an active event (started 1 hour ago, ends 3 hours from now)
        RegionEvent currentEvent = new RegionEvent(
                "EVENT-001",
                "GLOBAL",
                now - (1000 * 60 * 60),     // 1 hour ago
                now + (1000 * 60 * 60 * 3), // 3 hours from now
                "Global Treasure Hunt"
        );
        activeEvents.add(currentEvent);
        
        // Create an upcoming event (starts tomorrow, lasts 5 hours)
        RegionEvent upcomingEvent = new RegionEvent(
                "EVENT-002",
                "NORTH-AMERICA",
                now + (1000 * 60 * 60 * 24),     // 24 hours from now
                now + (1000 * 60 * 60 * 24 + 1000 * 60 * 60 * 5), // 29 hours from now
                "North America Vault Drop"
        );
        activeEvents.add(upcomingEvent);
    }
    
    /**
     * Inner class representing a regional event
     */
    public static class RegionEvent implements Serializable {
        private String eventId;
        private String regionId;
        private long startTime;
        private long endTime;
        private String name;
        
        public RegionEvent(String eventId, String regionId, long startTime, long endTime, String name) {
            this.eventId = eventId;
            this.regionId = regionId;
            this.startTime = startTime;
            this.endTime = endTime;
            this.name = name;
        }
        
        public String getEventId() {
            return eventId;
        }
        
        public String getRegionId() {
            return regionId;
        }
        
        public long getStartTime() {
            return startTime;
        }
        
        public long getEndTime() {
            return endTime;
        }
        
        public String getName() {
            return name;
        }
        
        @Override
        public String toString() {
            return name + " (" + new Date(startTime) + " to " + new Date(endTime) + ")";
        }
    }
}