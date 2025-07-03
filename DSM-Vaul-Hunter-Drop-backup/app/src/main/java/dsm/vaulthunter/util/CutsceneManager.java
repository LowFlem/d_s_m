package dsm.vaulthunter.util;

import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.util.Log;

import java.util.HashMap;
import java.util.Map;

import dsm.vaulthunter.controller.CutsceneActivity;

/**
 * Manager for handling cutscene playback and tracking
 */
public class CutsceneManager {
    private static final String TAG = "CutsceneManager";
    private static final String PREF_NAME = "dsm_cutscenes";
    
    // Cutscene types
    public static final String CUTSCENE_INTRO = "intro";
    public static final String CUTSCENE_MARAUDER_FIRST = "marauder_first";
    public static final String CUTSCENE_CITY_FIRST = "city_first";
    public static final String CUTSCENE_TREASURE_FIRST = "treasure_first";
    public static final String CUTSCENE_VAULT_FIRST = "vault_first";
    public static final String CUTSCENE_DSM_CONNECTION = "dsm_connection";
    
    // Singleton instance
    private static CutsceneManager instance;
    
    // Context
    private Context context;
    
    // Cutscene information
    private Map<String, CutsceneInfo> cutscenes;
    
    // SharedPreferences for tracking viewed cutscenes
    private SharedPreferences preferences;
    
    /**
     * Private constructor
     * @param context Application context
     */
    private CutsceneManager(Context context) {
        this.context = context.getApplicationContext();
        this.preferences = context.getSharedPreferences(PREF_NAME, Context.MODE_PRIVATE);
        initializeCutscenes();
    }
    
    /**
     * Get the singleton instance
     * @param context Context
     * @return CutsceneManager instance
     */
    public static synchronized CutsceneManager getInstance(Context context) {
        if (instance == null) {
            instance = new CutsceneManager(context);
        }
        return instance;
    }
    
    /**
     * Initialize cutscene information
     */
    private void initializeCutscenes() {
        cutscenes = new HashMap<>();
        
        // Add cutscene information - filename, description, skippable, priority
        cutscenes.put(CUTSCENE_INTRO, new CutsceneInfo(
                "dsm_intro.mp4", 
                "Welcome to DSM Vault Hunter", 
                true, 
                100));
        
        cutscenes.put(CUTSCENE_MARAUDER_FIRST, new CutsceneInfo(
                "dsm_marauder_encounter.mp4", 
                "First Marauder Encounter", 
                true, 
                80));
        
        cutscenes.put(CUTSCENE_CITY_FIRST, new CutsceneInfo(
                "dsm_city_arrival.mp4", 
                "Arriving at the City", 
                true, 
                70));
        
        cutscenes.put(CUTSCENE_TREASURE_FIRST, new CutsceneInfo(
                "dsm_treasure_discovery.mp4", 
                "First Treasure Discovery", 
                true, 
                60));
        
        cutscenes.put(CUTSCENE_VAULT_FIRST, new CutsceneInfo(
                "dsm_vault_discovery.mp4", 
                "First Vault Discovery", 
                true, 
                50));
        
        cutscenes.put(CUTSCENE_DSM_CONNECTION, new CutsceneInfo(
                "dsm_network_connection.mp4", 
                "Connecting to DSM Network", 
                true, 
                40));
    }
    
    /**
     * Check if a cutscene should be played
     * @param cutsceneId Cutscene identifier
     * @return true if the cutscene should be played
     */
    public boolean shouldPlayCutscene(String cutsceneId) {
        // Check if the cutscene exists
        if (!cutscenes.containsKey(cutsceneId)) {
            Log.e(TAG, "Unknown cutscene ID: " + cutsceneId);
            return false;
        }
        
        // Check if the cutscene has already been viewed
        return !preferences.getBoolean(getCutsceneKey(cutsceneId), false);
    }
    
    /**
     * Play a cutscene
     * @param cutsceneId Cutscene identifier
     * @return true if the cutscene was played
     */
    public boolean playCutscene(String cutsceneId) {
        if (!cutscenes.containsKey(cutsceneId)) {
            Log.e(TAG, "Unknown cutscene ID: " + cutsceneId);
            return false;
        }
        
        CutsceneInfo info = cutscenes.get(cutsceneId);
        
        // Start cutscene activity
        Intent intent = new Intent(context, CutsceneActivity.class);
        intent.putExtra(CutsceneActivity.EXTRA_CUTSCENE_ID, cutsceneId);
        intent.putExtra(CutsceneActivity.EXTRA_CUTSCENE_FILENAME, info.getFilename());
        intent.putExtra(CutsceneActivity.EXTRA_CUTSCENE_DESCRIPTION, info.getDescription());
        intent.putExtra(CutsceneActivity.EXTRA_CUTSCENE_SKIPPABLE, info.isSkippable());
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        
        context.startActivity(intent);
        
        // Mark as viewed
        markCutsceneAsViewed(cutsceneId);
        
        return true;
    }
    
    /**
     * Play a cutscene if it hasn't been viewed yet
     * @param cutsceneId Cutscene identifier
     * @return true if the cutscene was played
     */
    public boolean playIfNotViewed(String cutsceneId) {
        if (shouldPlayCutscene(cutsceneId)) {
            return playCutscene(cutsceneId);
        }
        return false;
    }
    
    /**
     * Mark a cutscene as viewed
     * @param cutsceneId Cutscene identifier
     */
    public void markCutsceneAsViewed(String cutsceneId) {
        preferences.edit().putBoolean(getCutsceneKey(cutsceneId), true).apply();
    }
    
    /**
     * Reset cutscene viewed status
     * @param cutsceneId Cutscene identifier
     */
    public void resetCutscene(String cutsceneId) {
        preferences.edit().putBoolean(getCutsceneKey(cutsceneId), false).apply();
    }
    
    /**
     * Reset all cutscenes
     */
    public void resetAllCutscenes() {
        preferences.edit().clear().apply();
    }
    
    /**
     * Get the preference key for a cutscene
     * @param cutsceneId Cutscene identifier
     * @return Preference key
     */
    private String getCutsceneKey(String cutsceneId) {
        return "cutscene_viewed_" + cutsceneId;
    }
    
    /**
     * Check if a cutscene file exists
     * @param cutsceneId Cutscene identifier
     * @return true if the cutscene file exists
     */
    public boolean cutsceneExists(String cutsceneId) {
        if (!cutscenes.containsKey(cutsceneId)) {
            return false;
        }
        
        CutsceneInfo info = cutscenes.get(cutsceneId);
        String filename = info.getFilename();
        
        try {
            String[] assets = context.getAssets().list("videos");
            for (String asset : assets) {
                if (asset.equals(filename)) {
                    return true;
                }
            }
        } catch (Exception e) {
            Log.e(TAG, "Error checking cutscene file: " + e.getMessage());
        }
        
        return false;
    }
    
    /**
     * Inner class for cutscene information
     */
    public class CutsceneInfo {
        private String filename;
        private String description;
        private boolean skippable;
        private int priority; // Higher priority cutscenes play first
        
        public CutsceneInfo(String filename, String description, boolean skippable, int priority) {
            this.filename = filename;
            this.description = description;
            this.skippable = skippable;
            this.priority = priority;
        }
        
        public String getFilename() {
            return filename;
        }
        
        public String getDescription() {
            return description;
        }
        
        public boolean isSkippable() {
            return skippable;
        }
        
        public int getPriority() {
            return priority;
        }
    }
}