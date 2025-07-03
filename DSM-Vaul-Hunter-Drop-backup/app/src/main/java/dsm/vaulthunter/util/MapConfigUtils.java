package dsm.vaulthunter.util;

import android.content.Context;
import android.os.Environment;
import android.util.Log;

import org.osmdroid.config.Configuration;

import java.io.File;

/**
 * Utility class for OpenStreetMap configuration
 */
public class MapConfigUtils {
    private static final String TAG = "MapConfigUtils";
    
    /**
     * Initialize the osmdroid configuration
     * @param context Application context
     */
    public static void initializeOsmDroid(Context context) {
        // Initialize osmdroid configuration
        Configuration.getInstance().load(context, 
                context.getSharedPreferences("osmdroid", Context.MODE_PRIVATE));
        
        // Set a custom user agent to identify the app
        Configuration.getInstance().setUserAgentValue("dsm-vault-hunter");
        
        // Configure tiles cache location
        configureTileCache(context);
    }
    
    /**
     * Configure the tile cache location
     * @param context Application context
     */
    private static void configureTileCache(Context context) {
        // Try to use external storage if available
        if (Environment.getExternalStorageState().equals(Environment.MEDIA_MOUNTED)) {
            try {
                // Get the app's private external storage directory
                File osmdroidDir = new File(context.getExternalFilesDir(null), "osmdroid");
                if (!osmdroidDir.exists()) {
                    boolean created = osmdroidDir.mkdirs();
                    if (!created) {
                        Log.e(TAG, "Failed to create osmdroid directory");
                    }
                }
                
                // Set the tile cache path
                Configuration.getInstance().setOsmdroidTileCache(osmdroidDir);
                Log.d(TAG, "Using external storage for tile cache: " + osmdroidDir.getAbsolutePath());
            } catch (Exception e) {
                Log.e(TAG, "Error setting external tile cache: " + e.getMessage());
                useInternalCache(context);
            }
        } else {
            // Fall back to internal storage
            useInternalCache(context);
        }
    }
    
    /**
     * Use internal storage for tile cache
     * @param context Application context
     */
    private static void useInternalCache(Context context) {
        try {
            // Use the app's internal cache directory
            File cacheDir = context.getCacheDir();
            File osmdroidDir = new File(cacheDir, "osmdroid");
            if (!osmdroidDir.exists()) {
                boolean created = osmdroidDir.mkdirs();
                if (!created) {
                    Log.e(TAG, "Failed to create internal osmdroid directory");
                }
            }
            
            // Set the tile cache path
            Configuration.getInstance().setOsmdroidTileCache(osmdroidDir);
            Log.d(TAG, "Using internal storage for tile cache: " + osmdroidDir.getAbsolutePath());
        } catch (Exception e) {
            Log.e(TAG, "Error setting internal tile cache: " + e.getMessage());
        }
    }
    
    /**
     * Convert a distance in meters to an appropriate display string
     * @param distanceInMeters Distance in meters
     * @return Formatted distance string (e.g., "1.2 km" or "350 m")
     */
    public static String formatDistance(float distanceInMeters) {
        if (distanceInMeters >= 1000) {
            return String.format("%.1f km", distanceInMeters / 1000);
        } else {
            return String.format("%d m", Math.round(distanceInMeters));
        }
    }
}