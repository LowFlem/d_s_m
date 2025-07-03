package dsm.vaulthunter.util;

import android.app.Application;
import android.content.Context;
import android.content.SharedPreferences;
import android.util.Log;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.DSMClient;

/**
 * Application class for global application state
 */
public class AppContext extends Application {
    private static final String TAG = "VH_AppContext";
    
    // Shared preferences constants
    private static final String PREFS_NAME = "VaultHunterPrefs";
    private static final String PREF_USER_EMAIL = "user_email";
    private static final String PREF_USER_PASSWORD = "user_password";
    
    // Static reference to the application context
    private static Context appContext;
    
    @Override
    public void onCreate() {
        super.onCreate();
        
        // Store the application context for static access
        appContext = getApplicationContext();
        
        // Initialize controllers and services
        initializeApp();
    }
    
    /**
     * Get the application context
     * @return Application context
     */
    public static Context getAppContext() {
        return appContext;
    }
    
    /**
     * Initialize the application components
     */
    private void initializeApp() {
        Log.d(TAG, "Initializing Vault Hunter application");
        
        // Initialize database
        DatabaseHelper.getInstance(this);
        
        // Initialize the app controller singleton
        AppController.getInstance();
        
        // Check for saved login credentials
        tryAutoLogin();
    }
    
    /**
     * Try to automatically log in using saved credentials
     */
    private void tryAutoLogin() {
        SharedPreferences prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
        String savedEmail = prefs.getString(PREF_USER_EMAIL, null);
        String savedPassword = prefs.getString(PREF_USER_PASSWORD, null);
        
        if (savedEmail != null && savedPassword != null) {
            // Try to authenticate using saved credentials
            Log.d(TAG, "Attempting auto-login for user: " + savedEmail);
            
            // Use the DatabaseHelper to authenticate user
            // In a real implementation, this would check if the user exists in the database
            // For now, we'll just log it
            Log.d(TAG, "Auto-login credentials found, would authenticate in full implementation");
        }
    }
    
    /**
     * Save login credentials for auto-login
     * @param email User email
     * @param password User password
     */
    public void saveLoginCredentials(String email, String password) {
        SharedPreferences prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
        SharedPreferences.Editor editor = prefs.edit();
        
        editor.putString(PREF_USER_EMAIL, email);
        editor.putString(PREF_USER_PASSWORD, password);
        editor.apply();
        
        Log.d(TAG, "Saved login credentials for: " + email);
    }
    
    /**
     * Clear saved login credentials
     */
    public void clearLoginCredentials() {
        SharedPreferences prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
        SharedPreferences.Editor editor = prefs.edit();
        
        editor.remove(PREF_USER_EMAIL);
        editor.remove(PREF_USER_PASSWORD);
        editor.apply();
        
        Log.d(TAG, "Cleared saved login credentials");
    }
    
    /**
     * Check if auto-login credentials are saved
     * @return true if credentials are saved
     */
    public boolean hasLoginCredentials() {
        SharedPreferences prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
        return prefs.contains(PREF_USER_EMAIL) && prefs.contains(PREF_USER_PASSWORD);
    }
    
    @Override
    public void onTerminate() {
        super.onTerminate();
        
        // Clean up resources
        cleanupResources();
    }
    
    /**
     * Clean up application resources
     */
    private void cleanupResources() {
        // Disconnect from DSM network
        if (DSMClient.getInstance().isConnected()) {
            DSMClient.getInstance().disconnect();
        }
    }
}